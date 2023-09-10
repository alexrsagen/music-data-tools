# Tools for music data
Tools for interacting with music data (Spotify, Apple Music)

## Import Spotify playlists to Apple Music
Imports Spotify GDPR data dump (my_spotify_data / MyData) playlists to Apple Music via [Apple Music API](https://developer.apple.com/documentation/applemusicapi/).

The tool does not require an Apple Developer account, as it uses the public client token from [music.apple.com](https://music.apple.com/) to access the API.

### Usage
```
Usage: music-data-tools [OPTIONS] <COMMAND>

Commands:
  import-spotify-gdpr-playlists-to-apple-music-api
  (aliases: spotify-playlist-to-apple, spta)
          Import Spotify GDPR data dump (my_spotify_data / MyData) playlists to Apple Music via API.

  help
          Print this message or the help of the given subcommand(s)

Options:
  -l, --log-level <LOG_LEVEL>      Log level [off|error|warn|info|debug|trace] [default: info]
  -c, --config-path <CONFIG_PATH>  [default: config.json]
  -h, --help                       Print help
  -V, --version                    Print version
```

For example (using `cargo`):
```
cargo run -- spta ".\my_spotify_data\MyData\Playlist1.json"
```

### Features
- Select which playlists you want to import
- Import multiple playlists at the same time
- Run "headless" (without user interaction)
- Does not require Apple Developer account

### Known issues/shortcomings
- Will sometimes pick the wrong track, as the tool currently searches for `artist - track title` in the Apple Music catalog and naively picks the first result. It assumes the first result is correct and does not double check or verify anything. This can be improved.
