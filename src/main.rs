use reqwest;
use serde_json::Value;
use serde_yaml;
use std::fs::File;
use std::io::prelude::*;

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

    let response = reqwest::get(&url).await?.text().await?;

    let v: Value = serde_json::from_str(&response)?;

    match v["recenttracks"]["track"][0]["@attr"]["nowplaying"].as_str() {
        Some("true") => println!("User is currently listening to: {}", v["recenttracks"]["track"][0]["name"]),
        _ => println!("User is not listening to anything right now"),
    }

    Ok(())
}