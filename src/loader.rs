use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct TextSource {
    pub content: String,
    pub duration: f32,
}

#[derive(Deserialize, Debug, Resource, Default)]
pub struct Config {
    pub text_size: f32,
    pub window_width: f32,
    pub camera_offset: f32,
}

pub fn load_csv(file: &str) -> Result<Vec<TextSource>, Box<dyn Error>> {
    let mut csv_path = std::env::home_dir().unwrap();
    csv_path.push("ebb/presets/".to_string() + file);
    let file_content = std::fs::read_to_string(csv_path)?;
    println!("{}", file_content);

    let rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_content.as_bytes());

    let result: Vec<TextSource> = rdr
        .into_deserialize()
        .collect::<Result<Vec<TextSource>, csv::Error>>()?;
    Ok(result)
}

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    let mut conf_path = std::env::home_dir().unwrap();
    conf_path.push("ebb/config.toml");
    let file_content = std::fs::read_to_string(conf_path).unwrap();
    println!("{}", file_content);
    let result: Config = toml::from_str(&file_content.as_str())?;
    Ok(result)
}

pub fn unwrap_csv(f: &str) -> Vec<TextSource> {
    match load_csv(f) {
        Ok(n) => return n,
        Err(_e) => {
            println!("Err: Can't Load Preset File: {}", f);
            return vec![TextSource {
                content: "This is a Demo Text".to_string(),
                duration: 5.0,
            }];
        }
    };
}

pub fn unwrap_conf() -> Config {
    match load_config() {
        Ok(n) => return n,
        Err(_e) => {
            println!("Err: Can't Load Config File!");
            return Config {
                text_size: 1080.0,
                window_width: 1920.0,
                camera_offset: 0.0,
            };
        }
    };
}
