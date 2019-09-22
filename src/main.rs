use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer};
use actix_files::NamedFile;
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, Addr};

use std::path::{Path, PathBuf};
use std::thread;
use std::collections::{HashSet, HashMap};

use vlc;

use crossbeam_channel;

use regex::Regex;
use lazy_static;

// use serde::{Serialize};
use serde_json;

type BasicError = &'static str;

pub struct AppState {
    sender: crossbeam_channel::Sender<PlayerMessage>,
}

enum PlayerMessage {
    Play(u64),
    Pause,
    Resume,
    Stop,
    Register(Addr<PlayerWs>),
    Unregister(Addr<PlayerWs>),
}

impl std::convert::TryFrom<String> for PlayerMessage {
    type Error = &'static str;

    fn try_from(text: String) -> Result<Self, Self::Error> {

        lazy_static::lazy_static! {
            static ref RE_PLAY: Regex = Regex::new(r"Play;(\d+)").unwrap();
        }

        if text == "Pause" {
            Ok(PlayerMessage::Pause)
        } else if text == "Resume" {
            Ok(PlayerMessage::Resume)
        } else if text == "Stop" {
            Ok(PlayerMessage::Stop)
        } else if let Some(caps) = RE_PLAY.captures(&text) {
            if let Some(s) = caps.get(1) {
                // this unwrap should never fail, u64 compatibility is ensured by the regular expression
                Ok(PlayerMessage::Play(s.as_str().parse::<u64>().unwrap()))
            } else {
                Err("Cannot convert passed string to PlayerMessage.")
            }
        } else {
            Err("Cannot convert passed string to PlayerMessage.")
        }
    }
}


fn index(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = PathBuf::from("index.html");
    
    Ok(NamedFile::open(path)?)
}

fn controls(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = PathBuf::from("controls.js");
    
    Ok(NamedFile::open(path)?)
}

fn api_websocket((req, state): (HttpRequest, web::Data<AppState>), stream: web::Payload) -> actix_web::Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(PlayerWs{ sender: state.sender.clone() }, &req, stream);
    println!("{:?}", resp);
    resp
}

struct PlayerWs {
    sender: crossbeam_channel::Sender<PlayerMessage>,
}

impl Actor for PlayerWs {
    type Context = ws::WebsocketContext<Self>;
}

impl actix::Handler<WsOutgoingMsg> for PlayerWs {
    type Result = Result<(), BasicError>;
    fn handle(&mut self, msg: WsOutgoingMsg, ctx: &mut Self::Context) -> Self::Result {
        match msg.kind {
            WsOutGoingMsgKind::FsState(map) => ctx.text(serde_json::json!(map).to_string()),
            _ => ctx.text(msg.info)
        }
        Ok(())
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for PlayerWs {
    fn started(&mut self, ctx: &mut Self::Context) {
        // bring trait into scope for access to ctx.address()
        use actix::AsyncContext;

        let addr = ctx.address();
        match self.sender.send(PlayerMessage::Register(addr)) {
            Ok(()) => {
                ctx.text("Ws registered");
                println!("Ws registered");
            },
            Err(e) => {
                ctx.text("Ws registration failed");
                println!("Ws registration failed: {}", e);
            },
        }
    }
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        use std::convert::TryFrom;

        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                match PlayerMessage::try_from(text) {
                    Ok(player_msg) => {
                        match self.sender.send(player_msg){
                            Ok(()) => ctx.text("Ok"),
                            Err(_) => ctx.text("Err: Failed to pass message to player")
                        }
                    },
                    Err(_) => ctx.text("Err: Invalid message"),
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
        match self.sender.send(PlayerMessage::Unregister(addr)) {
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

#[derive(Clone, Debug)]
struct WsOutgoingMsg {
    info: String,
    kind: WsOutGoingMsgKind,
}

#[derive(Clone, Debug)]
enum WsOutGoingMsgKind {
    Play,
    Pause,
    Resume,
    Stop,
    FsChange,
    FsState(HashMap<u64, String>)
}

impl actix::Message for WsOutgoingMsg {
    type Result = Result<(), BasicError>;
}


fn valid_directory(s: String) -> Result<(), String>{
    let path = Path::new(&s);
    if path.is_dir() {
        Ok(())
    } else {
        Err(String::from("Not a valid path to a directory"))
    }
    
}

fn parse_media_dir(mut id: u64, path: &Path) -> Result<(u64, HashMap<u64, String>), std::io::Error>{
    let mut registered_media: HashMap<u64, String> = HashMap::new();

    lazy_static::lazy_static! {
        static ref RE_AUDIO_EXTENSION: Regex = Regex::new(r".+\.mp3").unwrap();
    }

    
    // expect will only fail when lacking permissions, existence and is_dir are guaranteed. TODO: properly handle this error.
    for entry in std::fs::read_dir(path)? {
        match entry {
            Ok(good_entry) => {
                if good_entry.path().is_dir() {
                    let (new_id, subdir_media) = parse_media_dir(id, &good_entry.path())?;
                    id = new_id;
                    registered_media.extend(subdir_media);
                } else {
                    // TODO: handle properly instead of unwrap
                    let path_str = good_entry.path().to_str().unwrap().to_string();
                    if RE_AUDIO_EXTENSION.is_match(good_entry.file_name().to_str().unwrap()) {
                        registered_media.insert(id, path_str);
                        id += 1;
                    } else {
                        println!("Ignoring file with unsupported file type in media directory: {}.", path_str)
                    }
                }
            },
            Err(e) => {
                println!("Failed to read a file in the media directory: {}",e )
            }
        }
    }
    Ok((id, registered_media))
}

fn broadcast(connections: &HashSet<Addr<PlayerWs>>, msg: WsOutgoingMsg) {
    for conn in connections {
        match conn.try_send(
            msg.clone()
        ){
            Ok(_) => {},
            Err(e) => {println!("Failed to send: {}", e)}
        }
    }
}


fn main() {

    let matches = clap::App::new("Fidelitas")
        .version("0.1")
        .author("Nicolas Mohr <Nico.Mohr@gmx.net")
        .about("Network audio player")
        .arg(clap::Arg::with_name("port")
            .takes_value(true)
            .default_value("8088")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("The port the server will listen on")
            )
        .arg(clap::Arg::with_name("dir")
            .takes_value(true)
            .default_value("./music")
            .short("d")
            .long("dir")
            .value_name("PATH")
            .help("The directory whose files will be available for playback.")
            .validator(valid_directory)
            )
        .get_matches();


    let path = PathBuf::from(matches.value_of("dir").unwrap());
    println!("Hosting files in folder: {}", path.to_str().unwrap());


    let port = matches.value_of("port").unwrap();
    println!("Listening on port: {}...", port);


    

    // sender will be passed to actix web as appstate
    // receiver will be passed to the global player thread, 
    let (sender, receiver) = crossbeam_channel::unbounded();

    let _handle = thread::spawn(move || {
        // player thread setup
        // TODO: move outside closure

        let vlc_instance = vlc::Instance::new().unwrap();
        let mediaplayer = vlc::MediaPlayer::new(&vlc_instance).unwrap();

        let mut ws_connections: HashSet<Addr<PlayerWs>> = HashSet::new();
        let (media_max_id, registered_media) = parse_media_dir(0, &path).expect("Unable to read media dir.");


        // channel handling loop
        loop {
            match receiver.recv() {
                Ok(msg) => {
                    match msg {
                        PlayerMessage::Play(track_id) => {
                            if let Some(track_path) = registered_media.get(&track_id) {
                                println!("Received track on worker thread: {}-{}", track_id, track_path);
                                let md = vlc::Media::new_path(&vlc_instance, track_path).unwrap();
                                mediaplayer.set_media(&md);
                                mediaplayer.play().unwrap();
                                broadcast(&ws_connections, WsOutgoingMsg{info: String::from("Somebody else played a new track."), kind: WsOutGoingMsgKind::Play});
                            } else {
                                println!("Received track request with invalid track_id: {}", track_id)
                            }                            
                        },
                        PlayerMessage::Pause => {
                            mediaplayer.pause();
                            broadcast(&ws_connections, WsOutgoingMsg{info: String::from("Somebody else paused."), kind: WsOutGoingMsgKind::Pause});
                        }, 
                        PlayerMessage::Resume => {
                            mediaplayer.play().unwrap();
                            broadcast(&ws_connections, WsOutgoingMsg{info: String::from("Somebody else resumed."), kind: WsOutGoingMsgKind::Resume});
                        },
                        PlayerMessage::Stop => {
                            mediaplayer.stop();
                            broadcast(&ws_connections, WsOutgoingMsg{info: String::from("Somebody else stopped."), kind: WsOutGoingMsgKind::Stop});
                        },
                        PlayerMessage::Register(ws) => {
                            ws_connections.insert(ws.clone());
                            match ws.try_send(WsOutgoingMsg{info: String::from("Initial connection, sending fs information"), kind: WsOutGoingMsgKind::FsState(registered_media.clone())}){
                                Ok(_) => {},
                                Err(e) => {println!("Failed to send FsChange message: {}", e)}
                            }
                        },
                        PlayerMessage::Unregister(ws) => {
                            ws_connections.remove(&ws);
                        },
                    }
                },
                Err(e) => println!("Recieved error on worker thread: {}", e),
            }
        }
    });

    let app_state = web::Data::new(AppState {
        sender: sender,
    });

    HttpServer::new(move || {
        App::new()
            .register_data(app_state.clone())
            .service(
                web::scope("player")
                    .route("", web::get().to(index))
            )
            .service(
                web::scope("api")
                    .route("ws", web::get().to(api_websocket))
            )
            .service(
                web::scope("static")
                    .route("controls.js", web::get().to(controls))
            )

    })
    .bind(format!("0.0.0.0:{}", port))
    .unwrap()
    .run()
    .unwrap();
}
