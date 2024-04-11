# Activitywatch watcher for last.fm

This is a simple activitywatch watcher for last.fm scrobble data. It uses the last.fm API to fetch scrobbles and sends them to the activitywatch server.

## Prerequisites

- [Activitywatch](https://github.com/ActivityWatch/activitywatch)
- [Rust](https://www.rust-lang.org/tools/install)
- [Last.fm API account](https://www.last.fm/)


## Installation

Clone the repository

```bash
git clone https://github.com/brayo-pip/aw-watcher-lastfm.git
```

cd into the directory

```bash
cd aw-watcher-lastfm
```


On first run, you will be prompted to configure last.fm API key and your last.fm username. You can get the apikey from the [last.fm API page](https://www.last.fm/api/account/create).

```bash
cargo run
```

This should take a few seconds then the events should be visible in localhost:5600. If aw-server or aw-server-rust is running.

If everything works as expected, you can build the binary and set up a systemd service to run it in the background.

```bash
cargo build --release
```

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.
