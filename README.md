# Activitywatch watcher for last.fm scrobbles

This is a simple activitywatch watcher for last.fm scrobble data. It uses the last.fm API to fetch scrobbles and sends them to the activitywatch server.

## Prerequisites

- [Activitywatch](https://github.com/ActivityWatch/activitywatch)
- [Rust](https://www.rust-lang.org/tools/install)
- [Last.fm API key](https://www.last.fm/api/account/create)


## Installation

Clone the repository

```bash
git clone https://github.com/brayo-pip/aw-watcher-lastfm.git
```
cd into the directory

```bash
cd aw-watcher-lastfm
```
quick test to see if it works.

Paste your api key in the config.yaml file.


```bash
cargo run
```
This should take a few a few seconds then the events should be visible in localhost:5600. If aw-server or aw-server-rust is running.


If everything works as expected, you can build the binary and set up a systemd service to run it in the background.

```bash
cargo build --release
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
