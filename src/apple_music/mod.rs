use std::fmt;

use serde::{Deserialize, Serialize};

mod client;
pub use client::Client;

pub mod request;
pub mod response;

pub trait ToRequestObject {
    fn id(&self) -> &str;
    fn object_type(&self) -> &ObjectType;

    fn to_request_object(&self) -> request::Object {
        request::Object {
            id: self.id().to_string(),
            object_type: self.object_type().clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentRating {
    #[default]
    NoRating,
    Clean,
    Explicit,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObjectType {
    Albums,
    LibraryAlbums,
    Artists,
    LibraryArtists,
    Songs,
    LibrarySongs,
    MusicVideos,
    LibraryMusicVideos,
    Playlists,
    LibraryPlaylists,
    Stations,
    Ratings,
    Genres,
    Activities,
    Curators,
    RecordLabels,
    PersonalRecommendation,
}

impl ObjectType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Albums => "albums",
            Self::LibraryAlbums => "library-albums",
            Self::Artists => "artists",
            Self::LibraryArtists => "library-artists",
            Self::Songs => "songs",
            Self::LibrarySongs => "library-songs",
            Self::MusicVideos => "music-videos",
            Self::LibraryMusicVideos => "library-music-videos",
            Self::Playlists => "playlists",
            Self::LibraryPlaylists => "library-playlists",
            Self::Stations => "stations",
            Self::Ratings => "ratings",
            Self::Genres => "genres",
            Self::Activities => "activities",
            Self::Curators => "curators",
            Self::RecordLabels => "record-labels",
            Self::PersonalRecommendation => "personal-recommendation",
        }
    }
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioVariant {
    DolbyAtmos,
    DolbyAudio,
    HiResLossless,
    Lossless,
    LossyStereo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlaylistType {
    Editorial,
    External,
    PersonalMix,
    Replay,
    UserShared,
}
