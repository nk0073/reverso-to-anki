use std::{
    fs::OpenOptions,
    io::{Read, Write},
    process::exit,
};

use serde::Deserialize;

use crate::utils::get_path;

const CONFIG_FILE_NAME: &str = "cfg.toml";
const DEFAULT_CONFIG: &str = r#"
language = 'en'
anki_file_name = 'definitions.apkg'
port = 4444

[model]
id = 737373737373
name = 'Model'

[deck]
id = 73737373737373
name = 'Definitions'
description = 'Word definitions'
"#;

#[derive(Deserialize)]
pub struct Config {
    pub deck: DeckConfig,
    pub model: ModelConfig,
    pub language: String,
    pub anki_file_name: String,
    pub port: u32,
}

#[derive(Deserialize)]
pub struct ModelConfig {
    pub id: i64,
    pub name: String,
}

#[derive(Deserialize)]
pub struct DeckConfig {
    pub id: i64,
    pub name: String,
    pub description: String,
}

pub fn get_config() -> Config {
    let path = get_path(CONFIG_FILE_NAME);
    let mut file_created = false;
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .read(true)
        .open(&path)
        .or_else(|err| {
            if err.kind() == std::io::ErrorKind::NotFound {
                println!("Config file not found. Generating a default {CONFIG_FILE_NAME}");
                file_created = true;
                let mut file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(&path)
                    .unwrap();
                file.write(DEFAULT_CONFIG.as_bytes()).unwrap();
                println!("Config file created, you should change it to your liking and start the program again.");
                exit(0);
            } else {
                Err(err)
            }
        })
        .expect("Couldn't open or create the config file. Panicking...");

    let mut text = DEFAULT_CONFIG.to_string();
    if !file_created {
        file.read_to_string(&mut text).unwrap();
    }

    toml::from_str::<Config>(&text[..]).unwrap()
}
