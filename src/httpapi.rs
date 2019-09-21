use actix_web::{web, HttpRequest, Responder};
use crate::{PlayerMessage, AppState};


pub fn play((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {

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

pub fn pause((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {
    
    match state.sender.send(PlayerMessage::Pause) {
        Ok(_) => {
            format!("Pause message sent successfully")
        },
        Err(e) => {
            format!("Failed to send pause message: {}", e)
        }
    }
}

pub fn stop((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {
    
    match state.sender.send(PlayerMessage::Stop) {
        Ok(_) => {
            format!("Stop message sent successfully")
        },
        Err(e) => {
            format!("Failed to send pause message: {}", e)
        }
    }
}

pub fn resume((_req, state): (HttpRequest, web::Data<AppState>)) -> impl Responder {
    
    match state.sender.send(PlayerMessage::Resume) {
        Ok(_) => {
            format!("Pause message sent successfully")
        },
        Err(e) => {
            format!("Failed to send pause message: {}", e)
        }
    }
}