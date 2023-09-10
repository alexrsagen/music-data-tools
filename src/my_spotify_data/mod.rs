use std::fmt;

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Playlists {
	pub playlists: Vec<Playlist>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistTrack {
	pub track_name: String,
	pub artist_name: String,
	pub album_name: String,
	pub track_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistEpisode {
	pub episode_name: String,
	pub show_name: String,
	pub episode_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlaylistLocalTrack {
	pub uri: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum PlaylistItemInner {
	Track {
		track: PlaylistTrack,
	},
	Episode {
		episode: PlaylistEpisode,
	},
	#[serde(rename_all = "camelCase")]
	LocalTrack {
		local_track: PlaylistLocalTrack,
	},
}

#[derive(Debug, Clone)]
pub enum PlaylistItemAbstraction<'a> {
	Track(&'a PlaylistTrack),
	Episode(&'a PlaylistEpisode),
	LocalTrack(&'a PlaylistLocalTrack),
}

impl<'a> From<&'a PlaylistItemInner> for PlaylistItemAbstraction<'a> {
	fn from(value: &'a PlaylistItemInner) -> Self {
		match value {
			PlaylistItemInner::Track { track } => Self::Track(track),
			PlaylistItemInner::Episode { episode } => Self::Episode(episode),
			PlaylistItemInner::LocalTrack { local_track } => Self::LocalTrack(local_track),
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistItem {
	#[serde(flatten)]
	item: PlaylistItemInner,
	pub added_date: NaiveDate,
}

impl PlaylistItem {
	pub fn item(&self) -> PlaylistItemAbstraction {
		(&self.item).into()
	}
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
	pub name: String,
	pub last_modified_date: NaiveDate,
	pub items: Vec<PlaylistItem>,
	pub description: Option<String>,
	pub number_of_followers: u64,
}

impl fmt::Display for Playlist {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.name)
	}
}