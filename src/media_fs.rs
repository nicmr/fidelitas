/// Player filesystem utilities and events
/// 

use std::collections::{HashMap, HashSet};
use std::path::Path;

use regex::{Regex};
use vlc::Media;


/// Describes the files that are recognized as media files
pub struct ParseMediaConfig {
    extension_re : Regex,
}
impl ParseMediaConfig {
    pub fn new(file_extensions: &HashSet<&str>) -> Self {
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

/// Parses the files recognized as media files according to the ParseMediaConfig in the specified directory
pub fn parse_media_dir(mut min_id: u64, path: &Path, vlc_instance: &vlc::Instance, config: &ParseMediaConfig) -> Result<(u64, HashMap<u64, (String, Media)>), std::io::Error>{
    let mut registered_media: HashMap<u64, (String, Media)> = HashMap::new();    
    for entry in std::fs::read_dir(path)? {
        match entry {
            Ok(good_entry) => {
                if good_entry.path().is_dir() {
                    // recurse into subdirectory
                    // TODO: handle result instead of escalating with ?
                    let (new_id, subdir_media) = parse_media_dir(min_id, &good_entry.path(), vlc_instance,config)?;
                    min_id = new_id;
                    registered_media.extend(subdir_media);
                } else {
                    // TODO: handle properly instead of expect
                    let path_str = good_entry
                        .path()
                        .to_str()
                        .expect("Failed to convert music folder subpath to string. This is a bug.")
                        .to_string();

                    if config.extension_re.is_match(good_entry.file_name().to_str().expect("Failed to convert filename in music folder to string. This is a bug.")) {
                        if let Some(vlc_media) = vlc::Media::new_path(vlc_instance, good_entry.path()) {
                            registered_media.insert(min_id, (path_str, vlc_media));
                            min_id += 1;
                        } else {
                            println!("Found media file candidate but vlc failed to read the file.")
                        }
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
    Ok((min_id, registered_media))
}