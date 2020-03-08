/// This module contains all messages that will be serialized and/or deserialized and transmitted
/// over the websocket connection with the frontend
/// As well as an actix actor implementation to send an receive these messages
use std::collections::{HashMap};

use serde::{Serialize, Deserialize};
use actix::{StreamHandler, Actor};
use actix_web_actors::ws;

use crate::vlc_helpers;
use crate::{PlayerMsg};

#[derive(Clone, Debug, Deserialize)]
#[serde(tag="type")]
pub enum IncomingMsg {
    VolumeChange {volume: u64},
    Play {track_id: u64},
    Pause,
    Stop,
    Resume,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag="type")]
pub enum OutgoingMsg {
    // Play{id: u64, length: i64},
    // Pause,
    // Resume,
    // Stop,
    FsChange,
    PlaybackChange{playback_state : PlaybackState},
    PlayerState{playback_state: PlaybackState, media: HashMap<u64, String>}, //change type of media to MediaMetadata
    RegisterSuccess,
    Error,
    VolumeChange{volume: u64}
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct CurrentMedia {
    id: u64,
    length: i64,
    progress: i64
}

impl CurrentMedia {
    pub fn new (media_id : u64, mediaplayer: &vlc::MediaPlayer) -> Self {
        let media_length = unsafe { vlc_helpers::current_track_length(mediaplayer)};
        if let Some(media_progress) =  mediaplayer.get_time() {
            CurrentMedia {
                id: media_id,
                length: media_length,
                progress: media_progress
            }
        }
        else {
            println!("Failed to get media time progress. Defaulting to 0");
            CurrentMedia {
                id: media_id,
                length: media_length,
                progress: 0,
            }
        }
        
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(tag="playback-type")]
pub enum PlaybackState {
    Playing{ current_media: CurrentMedia},
    Paused{ current_media: CurrentMedia},
    Stopped,
}

impl PlaybackState {
    pub fn new (media_id: u64, mediaplayer: &vlc::MediaPlayer) -> Self {
        match mediaplayer.state() {
            vlc::State::Playing => {
                
                PlaybackState::Playing { current_media: CurrentMedia::new(media_id, mediaplayer)}
            },
            vlc::State::Paused => {
                PlaybackState::Paused { current_media: CurrentMedia::new(media_id, mediaplayer)}
            },
            vlc::State::Stopped => { 
                PlaybackState::Stopped
            }
            _ => {  // TODO: add custom handling for different player states
                PlaybackState::Stopped
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct MediaMetadata {
    title: String,
    artist: String,
    album: String,
}



type BasicError = &'static str;



pub struct PlayerWs {
    pub sender: crossbeam_channel::Sender<PlayerMsg>,
}

impl Actor for PlayerWs {
    type Context = ws::WebsocketContext<Self>;
}

impl actix::Handler<OutgoingMsg> for PlayerWs {
    type Result = Result<(), BasicError>;
    fn handle(&mut self, msg: OutgoingMsg, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(serde_json::json!(msg).to_string());
        Ok(())
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for PlayerWs {
    fn started(&mut self, ctx: &mut Self::Context) {
        // bring trait into scope for access to ctx.address()
        use actix::AsyncContext;

        let addr = ctx.address();
        match self.sender.send(PlayerMsg::Register(addr)) {
            Ok(()) => {
                println!("Ws registered");
            },
            Err(e) => {
                println!("Ws registration failed: {}", e);
            },
        }
    }
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {

        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                println!("received:{}", &text);
                let deserialized: serde_json::Result<IncomingMsg> = serde_json::from_str(&text);
                match deserialized {
                    Ok(msg) => {
                        let send_result = match msg {
                            IncomingMsg::VolumeChange{volume} => self.sender.send(PlayerMsg::VolumeChange(volume)),
                            IncomingMsg::Play{track_id} => self.sender.send(PlayerMsg::Play(track_id)),
                            IncomingMsg::Pause => self.sender.send(PlayerMsg::Pause),
                            IncomingMsg::Stop => self.sender.send(PlayerMsg::Stop),
                            IncomingMsg::Resume => self.sender.send(PlayerMsg::Resume),
                        };
                        match send_result {
                            Ok(()) => {
                                // everything good
                            }
                            Err(_) => {
                                // TODO: Retry? Notify client?
                            }
                        }
                    }
                    Err(_) => {
                        println!("Failed to deserialize message: '{}'", &text);
                        // TODO:inform client that message was not understood
                    },
                }
            }
            _ => (),
        }
    }
    fn finished(&mut self, ctx: &mut Self::Context) {
        //bring traits into scope for access to ctx.address() and ctx.stop()
        use actix::AsyncContext;
        use actix::ActorContext;

        let addr = ctx.address();
        match self.sender.send(PlayerMsg::Unregister(addr)) {
            Ok(()) => {
                println!("Ws unregistered");
            },
            Err(e) => {
                println!("Ws unregistration failed: {}", e);
            },
        }
        ctx.stop();
    }
}

impl actix::Message for OutgoingMsg {
    type Result = Result<(), BasicError>;
}