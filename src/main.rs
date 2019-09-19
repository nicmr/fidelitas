use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use actix_web_actors::ws;
use actix::{Actor, StreamHandler};

use std::path::PathBuf;
use vlc;
use std::thread;

use crossbeam_channel;

use lazy_static;



struct AppState {
    sender: crossbeam_channel::Sender<PlayerMessage>,
}

enum PlayerMessage {
    Play(String),
    Pause,
    Resume,
    Stop
}

impl std::convert::TryFrom<String> for PlayerMessage {
    type Error = &'static str;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        use regex::Regex;

        lazy_static::lazy_static! {
            static ref RE_PLAY: Regex = Regex::new(r"Play;(\w+)").unwrap();
        }

        if text == "Pause" {
            Ok(PlayerMessage::Pause)
        } else if text == "Resume" {
            Ok(PlayerMessage::Resume)
        } else if text == "Stop" {
            Ok(PlayerMessage::Stop)
        } else if let Some(caps) = RE_PLAY.captures(&text) {
            if let Some(s) = caps.get(1) {
                Ok(PlayerMessage::Play(s.as_str().to_string()))
            } else {
                Err("Cannot convert passed string to PlayerMessage.")
            }
        } else {
            Err("Cannot convert passed string to PlayerMessage.")
        }
    }
}


fn index(_req: HttpRequest) -> Result<NamedFile> {
    // let path: PathBuf = req.match_info().query("~/Code/rust/fidelitas/index.html").parse().unwrap();
    let path: PathBuf = PathBuf::from("index.html");
    
    Ok(NamedFile::open(path)?)
}

struct PlayerWs {
    sender: crossbeam_channel::Sender<PlayerMessage>,
}

impl Actor for PlayerWs {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<ws::Message, ws::ProtocolError> for PlayerWs {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        use std::convert::TryFrom;

        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => {
                match PlayerMessage::try_from(text) {
                    Ok(player_msg) => {
                        // TODO: handle send() result
                        self.sender.send(player_msg);
                        ctx.text("Ok");
                    },
                    Err(_) => ctx.text("Err"),
                }
            }
            _ => (),
        }
    }
}

fn api_websocket((req, state): (HttpRequest, web::Data<AppState>), stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(PlayerWs{ sender: state.sender.clone() }, &req, stream);
    println!("{:?}", resp);
    resp
}

// fn api_ws(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
//     let resp = ws::start();
//     resp
// }

fn api_play((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {

    let track_name = String::from("Xi-FreedomDive.mp3");

    match state.sender.send(PlayerMessage::Play(track_name.clone())) {
        Ok(_) => {
            format!("Track name sent succesfully: {}", track_name)
        },
        Err(e) => {
            format!("Failed to send track: {}", e)
        }
    }
}

fn api_pause((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {
    
    match state.sender.send(PlayerMessage::Pause) {
        Ok(_) => {
            format!("Pause message sent successfully")
        },
        Err(e) => {
            format!("Failed to send pause message: {}", e)
        }
    }
}

fn api_stop((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {
    
    match state.sender.send(PlayerMessage::Stop) {
        Ok(_) => {
            format!("Stop message sent successfully")
        },
        Err(e) => {
            format!("Failed to send pause message: {}", e)
        }
    }
}

fn api_resume((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {
    
    match state.sender.send(PlayerMessage::Resume) {
        Ok(_) => {
            format!("Pause message sent successfully")
        },
        Err(e) => {
            format!("Failed to send pause message: {}", e)
        }
    }
}

fn main() {

    let matches = clap::App::new("Fidelitas")
        .version("0.1")
        .author("Nicolas Mohr <Nico.Mohr@gmx.net")
        .about("Network audio player")
        .arg(clap::Arg::with_name("port")
            .default_value("8088")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Specifies the port the server will listen on")
            .takes_value(true))
        .get_matches();

    let port = matches.value_of("port").unwrap();
    println!("Listening on port: {}...", port);


    let (sender, receiver) = crossbeam_channel::unbounded();

    let _handle = thread::spawn(move || {
        let vlc_instance = vlc::Instance::new().unwrap();
        let mediaplayer = vlc::MediaPlayer::new(&vlc_instance).unwrap();
        loop {
            match receiver.recv() {
                Ok(msg) => {
                    match msg {
                        PlayerMessage::Play(track_name) => {
                            println!("Received track on worker thread: {}", track_name);
                            let md = vlc::Media::new_path(&vlc_instance, track_name).unwrap();
                            mediaplayer.set_media(&md);
                            mediaplayer.play().unwrap();
                        },
                        PlayerMessage::Pause => {
                            mediaplayer.pause();
                        },
                        PlayerMessage::Resume => {
                            mediaplayer.play().unwrap();
                        },
                        PlayerMessage::Stop => {
                            mediaplayer.stop();
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
                    .route("play/{file}", web::get().to(api_play))
                    .route("pause", web::get().to(api_pause))
                    .route("resume", web::get().to(api_resume))
                    .route("stop", web::get().to(api_stop))
                    .route("ws", web::get().to(api_websocket))
            )

    })
    .bind(format!("0.0.0.0:{}", port))
    .unwrap()
    .run()
    .unwrap();
}
