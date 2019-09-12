use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};

use std::path::PathBuf;
use vlc;
use std::thread;
// use std::sync::Mutex;

use crossbeam_channel;



struct AppState {
    // counter: Mutex<i32>,
    sender: crossbeam_channel::Sender<PlayerMessage>,
}

enum PlayerMessage {
    Play(String),
    Pause,
    Resume,
    Stop
}


fn index(_req: HttpRequest) -> Result<NamedFile> {
    // let path: PathBuf = req.match_info().query("~/Code/rust/fidelitas/index.html").parse().unwrap();
    let path: PathBuf = PathBuf::from("index.html");
    
    Ok(NamedFile::open(path)?)
}

fn api_play((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {
    // let command = req.match_info().query("command");
    // let file = req.match_info().query("file");
    // println!("about to {} file {}", command, file);

    // let vlc_instance = vlc::Instance::new().unwrap();
    // let md = vlc::Media::new_path(&vlc_instance, file).unwrap();
    // let mdp = vlc::MediaPlayer::new(&vlc_instance).unwrap();
    // mdp.set_media(&md);
    // mdp.play().unwrap();
    // thread::sleep(std::time::Duration::from_secs(10));

    // HttpResponse::Ok().body("success")

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




// fn api_test(data: web::Data<AppState>) -> String {
//     let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
//     *counter += 1; // <- access counter inside MutexGuard
//     format!("Request number: {}", counter)
// }



fn main() {
    let port = "8088";
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

     // TODO: use mpsc instead of mutex?
    let app_state = web::Data::new(AppState {
        // counter: Mutex::new(0),
        sender: sender,
    });

    HttpServer::new(move || {
        App::new()
            .register_data(app_state.clone())
            .service(
                web::scope("player")
                    .route("/", web::get().to(index))
            )
            .service(
                web::scope("api")
                    .route("play/{file}", web::get().to(api_play))
                    .route("pause", web::get().to(api_pause))
                    .route("resume", web::get().to(api_resume))
                    .route("stop", web::get().to(api_stop))
            )
    })
    .bind(format!("0.0.0.0:{}", port))
    .unwrap()
    .run()
    .unwrap();
}
