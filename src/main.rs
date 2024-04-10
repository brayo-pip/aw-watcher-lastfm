use reqwest;
use serde_json::{Value, Map};
use serde_yaml;
use tokio::time::sleep;
use std::fs::File;
use std::io::prelude::*;
use aw_client_rust::AwClient;
use aw_models::Event;
use chrono::{TimeDelta, Utc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("config.yaml").expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");
    let yaml: Value = serde_yaml::from_str(&contents).unwrap();
    let apikey = yaml["apikey"].as_str().unwrap().to_string();
    let username = yaml["username"].as_str().unwrap().to_string();
    if apikey == "your-api-key" || apikey == "" {
        panic!("Please set your API key in the config.yaml file");
    }

    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", username, apikey);

    let aw_client = AwClient::new("localhost", "5600", "aw-firebase-sync");
    // creates a new bucket if it doesn't exist, otherwise does nothing
    aw_client.create_bucket("aw-watcher-lastfm", "lastfm_events").unwrap();
    
    loop {
        let response = reqwest::get(&url).await?.text().await?;

        let v: Value = serde_json::from_str(&response)?;

        if v["recenttracks"]["track"][0]["@attr"]["nowplaying"].as_str() != Some("true") {
            sleep(std::time::Duration::from_secs(1)).await;
            continue;
        }
        let mut data = Map::new();
        data.insert("track".to_string(), v["recenttracks"]["track"][0]["name"].clone());
        data.insert("artist".to_string(), v["recenttracks"]["track"][0]["artist"]["#text"].clone());
        data.insert("album".to_string(), v["recenttracks"]["track"][0]["album"]["#text"].clone());
        let event = Event {
            id: None,
            timestamp: Utc::now(),
            duration: TimeDelta::seconds(1),
            data: data,
        };
        aw_client.heartbeat("aw-watcher-lastfm", &event, 5.0).unwrap();
        sleep(std::time::Duration::from_secs(1)).await;
    }
}
