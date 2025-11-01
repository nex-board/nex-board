use serde::{Serialize, Deserialize};
use std::error::Error;
use bevy::prelude::Resource;

#[derive(Serialize, Deserialize, Debug, Resource)]
pub struct TextSource {
    pub content: String,
    pub duration: f32,
}

pub fn load_csv(file: &str) -> Result<Vec<TextSource>, Box<dyn Error>> {
    let csv_path = String::from("../assets/presets/") + file;
    let file_content = std::fs::read_to_string(csv_path)?;
    println!("{}", file_content);

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_content.as_bytes());

    let result: Vec<TextSource> = rdr.into_deserialize()
        .collect::<Result<Vec<TextSource>, csv::Error>>()?;
    Ok(result)
}
