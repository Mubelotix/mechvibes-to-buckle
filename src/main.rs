use std::{collections::HashMap, process::Command};
use progress_bar::{finalize_progress_bar, inc_progress_bar, init_progress_bar_with_eta};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct SoundPackConfig {
    default: bool,
    defines: HashMap<usize, (usize, usize)>,
    id: String,
    includes_numpad: bool,
    key_define_type: String,
    name: String,
    sound: String,
    tags: Vec<String>,
}

impl std::fmt::Debug for SoundPackConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundPackConfig")
            .field("default", &self.default)
            .field("id", &self.id)
            .field("includes_numpad", &self.includes_numpad)
            .field("key_define_type", &self.key_define_type)
            .field("name", &self.name)
            .field("sound", &self.sound)
            .field("tags", &self.tags)
            .finish_non_exhaustive()
    }
}

fn main() {
    let sound_pack = std::env::var("SOUND_PACK").unwrap_or("1200000000001".to_string());
    
    // Download config
    let config_url = format!("https://mechvibes.com/sound-packs/sound-pack-{sound_pack}/dist/config.json");
    let rep = minreq::get(config_url).send().expect("Failed to fetch config.json");
    if rep.status_code != 200 {
        panic!("Failed to fetch config.json: {}", rep.status_code);
    }
    let body = rep.as_str().expect("Failed to read response body");
    let config: SoundPackConfig = serde_json::from_str(body).expect("Failed to parse config.json");
    println!("Config: {config:#?}");

    // Download sound file
    let sound_url = format!("https://mechvibes.com/sound-packs/sound-pack-{sound_pack}/dist/{}", config.sound);
    let rep = minreq::get(sound_url).send().expect("Failed to fetch sound file");
    if rep.status_code != 200 {
        panic!("Failed to fetch sound file: {}", rep.status_code);
    }
    let body = rep.as_bytes();
    std::fs::write(&config.sound, body).expect("Failed to write sound file");

    // Convert sounds
    std::fs::create_dir(format!("pack-{sound_pack}")).unwrap_or_default();
    init_progress_bar_with_eta(config.defines.len());
    for (key, (start, duration)) in config.defines.iter() {        
        let command = format!("sox {} pack-{sound_pack}/{:02x}-stereo.wav trim {:.03} {:.03}", config.sound, key, *start as f64 / 1000., *duration as f64 / 1000.);
        Command::new("/usr/bin/sh").arg("-c").arg(command).status().expect("Failed to run sox");

        let command = format!("sox pack-{sound_pack}/{:02x}-stereo.wav -c 1 pack-{sound_pack}/{:02x}-1.wav", key, key);
        Command::new("/usr/bin/sh").arg("-c").arg(command).status().expect("Failed to run sox");

        inc_progress_bar();
    }
    finalize_progress_bar();
}
