use actix_web::{web, App, HttpResponse, HttpRequest, HttpServer};
use actix_files::NamedFile;
use actix_web_actors::ws;
use actix::{Actor, StreamHandler, Addr};

use std::path::{Path, PathBuf};
use std::thread;
use std::collections::{HashSet, HashMap, VecDeque};

use vlc;

use crossbeam_channel;

use regex::Regex;
// use lazy_static;

mod network_interfaces;

use serde::{Serialize, Deserialize};
use serde_json;

type BasicError = &'static str;

pub struct AppState {
    sender: crossbeam_channel::Sender<PlayerMsg>,
}

enum PlayerMsg {
    Play(u64),
    Pause,
    Resume,
    Stop,
    Register(Addr<PlayerWs>),
    Unregister(Addr<PlayerWs>),
    VolumeChange(u64),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag="type")]
enum IncomingMsg {
    VolumeChange {volume: u64},
    Play {track_id: u64},
    Pause,
    Stop,
    Resume,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag="type")]
enum OutgoingMsg {
    Play,
    Pause,
    Resume,
    Stop,
    FsChange,
    FsState{media: HashMap<u64, String>},
    RegisterSuccess,
    Error,
    VolumeChange{volume: u64}
}


struct PlayerWs {
    sender: crossbeam_channel::Sender<PlayerMsg>,
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


fn valid_directory(s: String) -> Result<(), String>{
    let path = Path::new(&s);
    if path.is_dir() {
        Ok(())
    } else {
        Err(String::from("Not a valid path to a directory"))
    }
}

fn valid_port(port: String) -> Result<(), String>{
    match port.parse::<u32>() {
        Ok(port_numeric) => {
            if port_numeric < 65536 {
                Ok(())
            } else {
                Err(String::from("Port needs to be in range 0 - 65535"))
            }
        },
        Err(_) => {
            Err(format!("'{}' is not a valid port", port))
        }
    }

}

fn populate_html_template(ip: &str, port: &str) -> std::io::Result<()> {
    use std::fs;
    use std::io::prelude::*;
    
    let template = fs::read_to_string("./templates/index.html")?;
    let populated =
        template
            .replace("{{IP}}", ip)
            .replace("{{PORT}}", port);
    let mut file = fs::File::create("./static/index.html")?;
    file.write_all(populated.as_bytes())?;
    Ok(())
}


struct ParseMediaConfig {
    extension_re : regex::Regex,
}
impl ParseMediaConfig {
    fn new(file_extensions: &HashSet<&str>) -> Self {
        let len = file_extensions.len();
        // TODO: this can probably be written somewhat more efficiently by avoiding reallocation
        let extension_str = file_extensions.iter()
            .fold(String::with_capacity(len*4),|mut a, b| { a.push_str("|\\."); a.push_str(b); a});
        let re_audio_extension = Regex::new(&format!(".+({})", &extension_str[2..])).expect("Failed to parse audio extension regex. This is a bug.");
        Self {
            extension_re: re_audio_extension,
        }        
    }
}

fn parse_media_dir(mut id: u64, path: &Path, config: &ParseMediaConfig) -> Result<(u64, HashMap<u64, String>), std::io::Error>{
    let mut registered_media: HashMap<u64, String> = HashMap::new();    
    for entry in std::fs::read_dir(path)? {
        match entry {
            Ok(good_entry) => {
                if good_entry.path().is_dir() {
                    // TODO: handle result instead of escalating with ?
                    let (new_id, subdir_media) = parse_media_dir(id, &good_entry.path(), config)?;
                    id = new_id;
                    registered_media.extend(subdir_media);
                } else {
                    // TODO: handle properly instead of expect
                    let path_str = good_entry
                        .path()
                        .to_str()
                        .expect("Failed to convert music folder subpath to string. This is a bug.")
                        .to_string();

                    if config.extension_re.is_match(good_entry.file_name().to_str().expect("Failed to convert filename in music folder to string. This is a bug.")) {
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

fn broadcast(connections: &HashSet<Addr<PlayerWs>>, msgkind: OutgoingMsg) {
    for conn in connections {
        match conn.try_send(
            msgkind.clone()
        ){
            Ok(_) => {},
            Err(e) => {println!("Failed to broadcast: {}", e)}
        }
    }
}


fn index(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = PathBuf::from("./static/index.html");
    
    Ok(NamedFile::open(path)?)
}

fn controls(_req: HttpRequest) -> actix_web::Result<NamedFile> {
    let path: PathBuf = PathBuf::from("./static/controls.js");
    
    Ok(NamedFile::open(path)?)
}

fn api_websocket((req, state): (HttpRequest, web::Data<AppState>), stream: web::Payload) -> actix_web::Result<HttpResponse, actix_web::Error> {
    let resp = ws::start(PlayerWs{ sender: state.sender.clone() }, &req, stream);
    println!("{:?}", resp);
    resp
}


fn main() {

    let matches = clap::App::new("Fidelitas")
        .version("0.1")
        .author("Gwendolyn Mohr <gwen@gwenmohr.com>")
        .about("Network audio player")
        .arg(clap::Arg::with_name("port")
            .takes_value(true)
            .default_value("8088")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("The port the server will listen on")
            .validator(valid_port)
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
        .arg(clap::Arg::with_name("extension")
            .takes_value(true)
            .short("e")
            .long("extension")
            .value_name("FILE_EXTENSION")
            .help("Explicitly allow file extensions to be read by the program. May cause crashes if files cannot be decoded.")
            .multiple(true)
        )
        .arg(clap::Arg::with_name("interface")
            .long("interface")
            .takes_value(true)
            .value_name("NETWORK_INTERFACE_NAME")
            .help("Manually select the network interface users should access the application with.")
        )
        .get_matches();


    let path = PathBuf::from(matches.value_of("dir").expect("Can't retrieve cli matches of flag 'dir'. This is a bug."));
    println!("Hosting files in folder: {}", path.to_str().expect("Can't convert music folder path to string. This is a bug."));


    let port = matches.value_of("port").expect("Can't retrieve cli matches of flag 'port'. This is a bug.");

    // select network interface and address
    let interface_candidates = match network_interfaces::interfaces() {
        Some(candidates) => candidates,
        None => {
            eprintln!("Unable to detect network interfaces on your system.");
            std::process::exit(1);
        }
    };

    let selected_interface = match network_interfaces::select_network_interface(&interface_candidates, matches.value_of("interface")) {
        Some (i) => i,
        None => {
            eprintln!("Unable to autoselect a network interface. Please manually pass the --interface flag.\n Found the following interfaces:");
            interface_candidates.iter().for_each(|a| println!("{:?}", a));
            std::process::exit(1);
        }
    };

    let host_address = {
        let sorted_ips = selected_interface.ip_addresses
            .iter()
            .fold(VecDeque::new(), network_interfaces::v4_first);

        match sorted_ips.iter().next() {
            Some(ip) => ip.to_string(),
            None => {
                eprintln!("Unable to get ipv4 adress from selected network interfaces. Interface: {:?}", selected_interface);
                std::process::exit(1);
            }
        }
            

       
    };

    // populate html template
    match populate_html_template(&host_address, port) {
        Ok (()) => {
            println!("Populated html template");
        },
        Err (e) => {
            eprintln!("failed to populate html: {}", e);
            std::process::exit(1);
        }
    }


    let parse_media_config = {
        let mut extension_set : HashSet<&str> = HashSet::with_capacity(5);
        // add default extensions
        // TODO: instead read from a default settings file
        extension_set.insert("mp3");
        extension_set.insert("ogg");
        extension_set.insert("opus");
        extension_set.insert("wav");
        extension_set.insert("m4a");

        match matches.values_of("extension") {
            Some(a) => {
                let user_set : HashSet<&str> = a.collect();
                extension_set.reserve(user_set.len());
                user_set.iter().for_each(|a| {extension_set.insert(a);});
            },
            None => {
            }
        }

        // print summary of enabled file extensions
        {
            let mut extension_info = String::from("Enabled file extensions: ");
            for extension in &extension_set {
                extension_info.push_str(extension);
                extension_info.push_str(", ");
            }
            println!("{}", extension_info);
        }

        ParseMediaConfig::new(&extension_set)
    };

    // initialize the channel for communication with the libvlc handler thread
    // sender will be passed to actix web as appstate and can be safely shared across websocket handlers
    // receiver will be passed to the global player thread, 
    let (sender, receiver) = crossbeam_channel::unbounded();


    let _handle = thread::spawn(move || {
        // player thread setup
        // TODO: move outside closure

        let vlc_instance = vlc::Instance::new().expect("Failed to initialize vlc instance. This is a bug.");
        let mediaplayer = vlc::MediaPlayer::new(&vlc_instance).expect("Failed to create vlc media player from vlc instance. This is a bug.");

        let mut ws_connections: HashSet<Addr<PlayerWs>> = HashSet::new();
        let (_media_max_id, registered_media) = parse_media_dir(0, &path, &parse_media_config).expect("Unable to read media dir.");


        // channel handling loop
        loop {
            match receiver.recv() {
                Ok(msg) => {
                    match msg {
                        PlayerMsg::Play(track_id) => {
                            if let Some(track_path) = registered_media.get(&track_id) {
                                println!("Received track on worker thread: k:'{}' V:'{}'", track_id, track_path);
                                // TODO: handle resiliently instead of expect
                                let md = vlc::Media::new_path(&vlc_instance, track_path).expect("Failed to create vlc media from file path. This is a bug.");
                                mediaplayer.set_media(&md);
                                // TODO: handle resiliently instead of expect
                                mediaplayer.play().expect("Failed to play selected vlc media. This is a bug.");
                                broadcast(&ws_connections, OutgoingMsg::Play);
                            } else {
                                println!("Received track request with invalid track_id: {}", track_id)
                            }                            
                        },
                        PlayerMsg::Pause => {
                            mediaplayer.pause();
                            broadcast(&ws_connections, OutgoingMsg::Pause);
                        },
                        // TODO: send more specific error message to client
                        PlayerMsg::Resume => {
                            if mediaplayer.will_play() {
                                match mediaplayer.play() {
                                    Ok(()) => broadcast(&ws_connections, OutgoingMsg::Resume),
                                    Err(()) => broadcast(&ws_connections, OutgoingMsg::Error)
                                }
                            } else {
                                broadcast(&ws_connections, OutgoingMsg::Error)
                            }
                        },
                        PlayerMsg::Stop => {
                            mediaplayer.stop();
                            broadcast(&ws_connections, OutgoingMsg::Stop);
                        },
                        PlayerMsg::VolumeChange(volume) => {
                            use vlc::MediaPlayerAudioEx;
                            use std::convert::TryInto;

                            // TODO:handle resiliently instead of expect
                            // check value for i32 bounds (and limits given by vlc?)
                            match mediaplayer.set_volume(volume.try_into().expect("Failed to convert volume change message. This is a bug.")) {
                                Ok(()) => {
                                   broadcast(&ws_connections, OutgoingMsg::VolumeChange{volume: volume});
                                },
                                Err(()) => {
                                    // TODO: log? retry?
                                }
                            }
                        }
                        PlayerMsg::Register(ws) => {
                            ws_connections.insert(ws.clone());
                            match ws.try_send(OutgoingMsg::FsState{media: registered_media.clone()}){
                                Ok(_) => {},
                                Err(e) => {println!("Failed to send FsChange message: {}", e)}
                            }
                        },
                        PlayerMsg::Unregister(ws) => {
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

    println!("Listening on port: {}...", port);
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
    // TODO: match instead and print specific error.
    .expect("Failed to bind port. The port might be in use. Try and specify a free port manually with the -p flag.")
    .run()
    .expect("Failed to start actix system. This is a bug.");
}
