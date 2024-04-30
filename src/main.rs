use aw_client_rust::AwClient;
use aw_models::{Bucket, Event};
use chrono::{TimeDelta, Utc};
use dirs::config_dir;
use reqwest;
use serde_json::{Map, Value};
use serde_yaml;
use std::fs::{DirBuilder, File};
use std::io::prelude::*;
use tokio::time::{interval, timeout, Duration};
use log::{warn, info};
use env_logger::Env;

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
    
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);

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

    let mut config_file = File::open(config_path.clone()).expect("Unable to open file");
    let mut contents = String::new();
    config_file.read_to_string(&mut contents)
        .expect("Unable to read file");

    let yaml: Value = serde_yaml::from_str(&contents).expect("Unable to parse yaml from config file");
    let apikey = yaml["apikey"].as_str().expect("Unable to get api key from config file").to_string();
    let username = yaml["username"].as_str().expect("Unable to get username from config file").to_string();
    let polling_interval = yaml["polling_interval"].as_i64().expect("Unable to get polling interval from config file");

    drop(config_file);

    // user just ran the program for the first time
    if username == "your_username" || username == "" {
        panic!("Please set your username at {:?}", config_path);
    }
    if apikey == "your-api-key" || apikey == "" {
        panic!("Please set your api key at {:?}", config_path);
    }

    let url = format!("http://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user={}&api_key={}&format=json&limit=1", username, apikey);

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

    let polling_time = TimeDelta::seconds(polling_interval);
    let mut interval = interval(Duration::from_secs(polling_interval as u64));

    let client = reqwest::Client::new();
    
    loop {
        interval.tick().await;

        let response = timeout(
            Duration::from_secs(5),
            client.get(&url).send(),
        ).await;
        let v: Value = match response {
            Ok(Ok(response)) => {
                response.json().await.unwrap()},
            Ok(Err(e)) => {
                warn!("Unable to connect to last.fm: {}", e);
                continue;
            },
            Err(_) => {
                warn!("Request timed out");
                continue;
            }
        };

        if v["recenttracks"]["track"][0]["@attr"]["nowplaying"].as_str() != Some("true") {
            info!("No song is currently playing");
            continue;
        }
        let mut event_data: Map<String, Value> = Map::new();
        info!("Track: {} - {}", v["recenttracks"]["track"][0]["name"], v["recenttracks"]["track"][0]["artist"]["#text"]);
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
    }
}
