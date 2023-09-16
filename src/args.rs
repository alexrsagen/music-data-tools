use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
	/// Import Spotify GDPR data dump (my_spotify_data / MyData) playlists to Apple Music via API.
	#[clap(aliases = &["spotify-playlist-to-apple", "spta"])]
	ImportSpotifyGdprPlaylistsToAppleMusicApi {
		/// Path to MyData/Playlistsn.json file from Spotify GDPR export
		playlist_file: PathBuf,

		/// List of playlist names to include
		#[clap(long, num_args = 0..)]
		playlists: Option<Vec<String>>,

		/// If set, will not create playlist or add tracks
		#[clap(long)]
		dry: bool,

		/// Minimum score for a match (between 0.0 and 3.0)
		#[clap(long, default_value = "0.8")]
		min_score: f64,

		/// Limit of possible songs per search result
		#[clap(long, default_value = "10")]
		limit: usize,
	}
}

#[derive(Debug, Parser)]
#[clap(author, about, version)]
pub struct Args {
    /// Log level [off|error|warn|info|debug|trace]
    #[clap(long, short = 'l', default_value = "info")]
	pub log_level: log::LevelFilter,

	// Config file path (default file will be created if it does not exist)
	#[clap(long, short = 'c', default_value = "config.json")]
	pub config_path: PathBuf,

	#[clap(subcommand)]
	pub command: Command,
}

impl Args {
	pub fn parse() -> Self {
		Parser::parse()
	}
}