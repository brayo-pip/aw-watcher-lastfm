use reqwest;
use serde_json::{Value, Map};
use serde_yaml;
use tokio::time::sleep;
use std::fs::{File, DirBuilder};
use std::io::prelude::*;
use aw_client_rust::AwClient;
use aw_models::Event;
use chrono::{TimeDelta, Utc};
use dirs::config_dir;

fn get_config_path() -> Option<std::path::PathBuf> {
  config_dir().map(|mut path| {
    path.push("aw-watcher-lastfm");
    path.push("config.yaml");
    path
  })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = get_config_path().expect("Unable to get config path");
    
    if !config_path.exists() {
        DirBuilder::new().recursive(true).create(config_path.parent().unwrap()).expect("Unable to create directory");
        let mut file = File::create(&config_path).expect("Unable to create file");
        file.write_all(b"apikey: your-api-key\nusername: your_username\npolling_interval: 10").expect("Unable to write to file");
        println!("Please set your api key and username at {:?}", config_path.clone());
        return Ok(());
    }

    let mut file = File::open(config_path.clone()).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");
    
    let yaml: Value = serde_yaml::from_str(&contents).unwrap();
    let apikey = yaml["apikey"].as_str().unwrap().to_string();
    let username = yaml["username"].as_str().unwrap().to_string();
    let polling_interval = yaml["polling_interval"].as_i64().unwrap();

    if username == "your_username" || username == "" {
        panic!("Please set your username at {:?}", config_path.clone());
    }
    if apikey == "your-api-key" || apikey == "" {
        panic!("Please set your api key at {:?}", config_path.clone());
    }

    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", username, apikey);

    let aw_client = AwClient::new("localhost", "5600", "aw-firebase-sync");
    // creates a new bucket if it doesn't exist, otherwise does nothing
    aw_client.create_bucket("aw-watcher-lastfm", "currently-playing").unwrap();
    
    loop {
        let response = reqwest::get(&url).await?.text().await?;
        let v: Value = serde_json::from_str(&response)?;
        if v["recenttracks"]["track"][0]["@attr"]["nowplaying"].as_str() != Some("true") {
            sleep(std::time::Duration::from_secs(polling_interval as u64)).await;
            continue;
        }
        let mut data = Map::new();
        data.insert("track".to_string(), v["recenttracks"]["track"][0]["name"].to_owned());
        data.insert("artist".to_string(), v["recenttracks"]["track"][0]["artist"]["#text"].to_owned());
        data.insert("album".to_string(), v["recenttracks"]["track"][0]["album"]["#text"].to_owned());
        let event = Event {
            id: None,
            timestamp: Utc::now(),
            duration: TimeDelta::seconds(polling_interval),
            data: data,
        };
        aw_client.heartbeat("aw-watcher-lastfm", &event, 5.0).unwrap();
        sleep(std::time::Duration::from_secs(polling_interval as u64)).await;
    }
}
