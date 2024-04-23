use aw_client_rust::AwClient;
use aw_models::{Bucket, Event};
use chrono::{TimeDelta, Utc};
use dirs::config_dir;
use reqwest;
use serde_json::{Map, Value};
use serde_yaml;
use std::fs::{DirBuilder, File};
use std::io::prelude::*;
use tokio::time::sleep;

fn get_config_path() -> Option<std::path::PathBuf> {
    config_dir().map(|mut path| {
        path.push("activitywatch");
        path.push("aw-watcher-lastfm");
        path
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_dir = get_config_path().expect("Unable to get config path");
    let config_path = config_dir.join("config.yaml");

    if !config_dir.exists() {
        DirBuilder::new()
            .recursive(true)
            .create(config_dir)
            .expect("Unable to create directory");
        let mut file = File::create(&config_path).expect("Unable to create file");
        file.write_all(b"apikey: your-api-key\nusername: your_username\npolling_interval: 10")
            .expect("Unable to write to file");
        println!(
            "Please set your api key and username at {:?}",
            config_path.clone()
        );
        return Ok(());
    }

    let mut file = File::open(config_path.clone()).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let yaml: Value = serde_yaml::from_str(&contents).unwrap();
    let apikey = yaml["apikey"].as_str().unwrap().to_string();
    let username = yaml["username"].as_str().unwrap().to_string();
    let polling_interval = yaml["polling_interval"].as_i64().unwrap();

    drop(file);

    if username == "your_username" || username == "" {
        panic!("Please set your username at {:?}", config_path.clone());
    }
    if apikey == "your-api-key" || apikey == "" {
        panic!("Please set your api key at {:?}", config_path.clone());
    }

    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json", username, apikey);

    let aw_client = AwClient::new("localhost", 5600, "aw-watcher-lastfm-rust").unwrap();
    // creates a new bucket if it doesn't exist, otherwise does nothing
    aw_client
        .create_bucket(&Bucket {
            id: "aw-watcher-lastfm".to_string(),
            bid: None,
            _type: "currently-playing".to_string(),
            data: Map::new(),
            metadata: Default::default(),
            last_updated: None,
            hostname: "".to_string(),
            client: "aw-watcher-lastfm-rust".to_string(),
            created: None,
            events: None,
        })
        .await
        .unwrap();

    let polling_duration = std::time::Duration::from_secs(polling_interval as u64);
    let polling_time = TimeDelta::seconds(polling_interval);
    let client = reqwest::Client::new();

    loop {
        let v = client.get(&url).send().await?.json::<Value>().await?;

        if v["recenttracks"]["track"][0]["@attr"]["nowplaying"].as_str() != Some("true") {
            sleep(polling_duration).await;
            continue;
        }
        let mut event_data: Map<String, Value> = Map::new();
        event_data.insert(
            "title".to_string(),
            v["recenttracks"]["track"][0]["name"].to_owned(),
        );
        event_data.insert(
            "artist".to_string(),
            v["recenttracks"]["track"][0]["artist"]["#text"].to_owned(),
        );
        event_data.insert(
            "album".to_string(),
            v["recenttracks"]["track"][0]["album"]["#text"].to_owned(),
        );
        let event = Event {
            id: None,
            timestamp: Utc::now(),
            duration: polling_time,
            data: event_data,
        };
        aw_client
            .heartbeat("aw-watcher-lastfm", &event, polling_interval as f64)
            .await
            .unwrap();
        sleep(polling_duration).await;
    }
}
