use std::{collections::HashMap, process::Command};
use progress_bar::{finalize_progress_bar, inc_progress_bar, init_progress_bar_with_eta};
use serde::Deserialize;
use string_tools::get_all_between_strict;

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
    let url = "https://mechvibes.com/sound-packs/";
    let rep = minreq::get(url).send().expect("Failed to fetch sound packs");
    if rep.status_code != 200 {
        panic!("Failed to fetch sound packs: {}", rep.status_code);
    }
    let mut body: &str = rep.as_str().expect("Failed to read response body");
    let mut sound_packs = Vec::new();
    while let Some(sound) = get_all_between_strict(body, "/sound-packs/", "\"") {
        if !sound.is_empty() {
            sound_packs.push(sound);
        }
        let body_idx = sound.as_ptr() as usize - body.as_ptr() as usize + sound.len();
        body = &body[body_idx..];
    }
    println!("Sound packs: {:#?}", sound_packs);
   
    init_progress_bar_with_eta(sound_packs.len());
    for sound_pack in sound_packs {
        // Download config
        let config_url = format!("https://mechvibes.com/sound-packs/{sound_pack}/dist/config.json");
        let rep = minreq::get(config_url).send().expect("Failed to fetch config.json");
        if rep.status_code != 200 {
            println!("Failed to fetch config.json: {}", rep.status_code);
            inc_progress_bar();
            continue;
        }
        let body = rep.as_str().expect("Failed to read response body");
        let config = match serde_json::from_str::<SoundPackConfig>(body) {
            Ok(config) => config,
            Err(e) => {
                println!("Failed to parse config.json: {:#?}", e);
                inc_progress_bar();
                continue;
            }
        };
        println!("Config: {config:#?}");

        // Download sound file
        let sound_url = format!("https://mechvibes.com/sound-packs/{sound_pack}/dist/{}", config.sound);
        let rep = minreq::get(sound_url).send().expect("Failed to fetch sound file");
        if rep.status_code != 200 {
            panic!("Failed to fetch sound file: {}", rep.status_code);
        }
        let body = rep.as_bytes();
        std::fs::write(&config.sound, body).expect("Failed to write sound file");

        // Convert sounds
        std::fs::create_dir(format!("packs/{sound_pack}")).unwrap_or_default();
        for (key, (start, duration)) in config.defines.iter() {
            // Extract sound
            let command = format!("sox {} packs/{sound_pack}/{:02x}-stereo.wav trim {:.03} {:.03}", config.sound, key, *start as f64 / 1000., *duration as f64 / 1000.);
            Command::new("/usr/bin/sh").arg("-c").arg(command).status().expect("Failed to run sox");

            // Convert to mono
            let command = format!("sox packs/{sound_pack}/{:02x}-stereo.wav -c 1 packs/{sound_pack}/{:02x}-1.wav", key, key);
            Command::new("/usr/bin/sh").arg("-c").arg(command).status().expect("Failed to run sox");

            // Remove stereo
            std::fs::remove_file(format!("packs/{sound_pack}/{:02x}-stereo.wav", key)).unwrap_or_default();
        }

        inc_progress_bar();
    }
    finalize_progress_bar();
}
