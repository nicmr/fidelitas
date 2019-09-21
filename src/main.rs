use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer};
use actix_files::NamedFile;
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, Addr};

use std::path::{Path, PathBuf};
use vlc;
use std::thread;

use crossbeam_channel;

use lazy_static;


// Http API, deprecated in favour of websocket api
mod httpapi;


type BasicError = &'static str;

pub struct AppState {
    sender: crossbeam_channel::Sender<PlayerMessage>,
}

enum PlayerMessage {
    Play(String),
    Pause,
    Resume,
    Stop,
    Register(Addr<PlayerWs>)
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


fn index(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    // let path: PathBuf = req.match_info().query("~/Code/rust/fidelitas/index.html").parse().unwrap();
    let path: PathBuf = PathBuf::from("index.html");
    
    Ok(NamedFile::open(path)?)
}

fn controls(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    // let path: PathBuf = req.match_info().query("~/Code/rust/fidelitas/index.html").parse().unwrap();
    let path: PathBuf = PathBuf::from("controls.js");
    
    Ok(NamedFile::open(path)?)
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

        ctx.text(msg.text);
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
    fn finished(&mut self, ctx: &mut Self::Context) {
        //bring traits into scope for access to ctx.address() and ctx.stop()
        use actix::AsyncContext;
        use actix::ActorContext;

        let addr = ctx.address();
        match self.sender.send(PlayerMessage::Register(addr)) {
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


struct WsOutgoingMsg {
    text: String,
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

fn api_websocket((req, state): (HttpRequest, web::Data<AppState>), stream: web::Payload) -> actix_web::Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(PlayerWs{ sender: state.sender.clone() }, &req, stream);
    println!("{:?}", resp);
    resp
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


    let path = Path::new(matches.value_of("dir").unwrap());
    println!("Hosting files in folder: {}", path.to_str().unwrap());


    let port = matches.value_of("port").unwrap();
    println!("Listening on port: {}...", port);

    


    let (sender, receiver) = crossbeam_channel::unbounded();

    let _handle = thread::spawn(move || {
        let vlc_instance = vlc::Instance::new().unwrap();
        let mediaplayer = vlc::MediaPlayer::new(&vlc_instance).unwrap();

        let mut ws_connections: Vec::<Addr<PlayerWs>> = Vec::new();

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
                            for conn in &mut ws_connections {
                                match conn.try_send(
                                    WsOutgoingMsg{
                                        text: String::from("Somebody else paused.")
                                    }
                                ){
                                    Ok(_) => {},
                                    Err(_) => {println!("Failed to send: ")}
                                }
                            }
                        },
                        PlayerMessage::Resume => {
                            mediaplayer.play().unwrap();
                        },
                        PlayerMessage::Stop => {
                            mediaplayer.stop();
                        },
                        PlayerMessage::Register(ws) => {
                            ws_connections.push(ws);
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
                    .route("play/{file}", web::get().to(httpapi::play))
                    .route("pause", web::get().to(httpapi::pause))
                    .route("resume", web::get().to(httpapi::resume))
                    .route("stop", web::get().to(httpapi::stop))
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
