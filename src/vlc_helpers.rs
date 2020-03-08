/// The contents of this module add functionality still missing in the rust bindings to libvlc.
/// Some of these are unsafe.
/// They should eventually be replaced when the bindings get updated or be merged into the libvlc bindings themselves.
/// 
/// 

use vlc;

/// Necessary because the rust bindings to libvlc do not yet offer a corresponding abstraction.
/// How to avoid undefined behaviour:
/// Only reads data, doesn't write, so it should not cause undefined behaviour as long as your vlc::MediaPlayer instance is immutable.
pub unsafe fn current_track_length (mediaplayer: &vlc::MediaPlayer) -> vlc::sys::libvlc_time_t {
    vlc::sys::libvlc_media_player_get_length((&mediaplayer).raw())
}
