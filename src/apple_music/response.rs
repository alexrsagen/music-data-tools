use anyhow::Result;
use chrono::{NaiveDate, DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::{Client, ContentRating, ObjectType, AudioVariant, PlaylistType, ToRequestObject};

#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMeta {
    #[serde(default)]
    pub total: Option<usize>,
    #[serde(default)]
    pub results: Option<SearchResultsMeta>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Error {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub status: String,
    pub code: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub errors: Vec<Error>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuccessfulListResponse<T> {
    #[serde(default)]
    pub next: Option<String>,
    pub data: Vec<T>,
    pub meta: Option<ResponseMeta>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ListResponse<T> {
    Success(SuccessfulListResponse<T>),
    Error(ErrorResponse),
}

impl<T: DeserializeOwned> ListResponse<T> {
    #[allow(unused)]
    pub async fn next(&self, client: &Client) -> Result<Option<Self>> {
        if let Self::Success(success_res) = &self {
            if let Some(next_url) = &success_res.next {
                let res = client.get(next_url).await?;
                Ok(Some(res))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

impl<T: DeserializeOwned + Clone> ListResponse<T> {
    pub async fn all(self, client: &Client) -> Result<Vec<T>> {
        let mut data = Vec::new();
        let mut next_url = None;

        if let ListResponse::Success(mut success_res) = self {
            next_url = success_res.next.clone();
            data.append(&mut success_res.data);
        }

        while let Some(url) = &next_url {
            if let ListResponse::Success(mut success_res) = client.get(url).await? {
                next_url = success_res.next;
                data.append(&mut success_res.data);
            } else {
                next_url = None;
            }
        }

        Ok(data)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artwork {
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub url: String,
}

impl Artwork {
    #[allow(unused)]
    pub fn url_with_dimensions(&self, fallback_width: u64, fallback_height: u64) -> String {
        self.url
            .replace("{w}", self.width.unwrap_or(fallback_width).to_string().as_str())
            .replace("{h}", self.height.unwrap_or(fallback_height).to_string().as_str())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayParams {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub is_library: bool,
    #[serde(default)]
    pub reporting: bool,
    #[serde(default)]
    pub global_id: Option<String>,
    #[serde(default)]
    pub catalog_id: Option<String>,
    #[serde(default)]
    pub reporting_id: Option<String>,
    #[serde(default)]
    pub version_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Preview {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySongAttributes {
    #[serde(default)]
    pub album_name: Option<String>,
    pub artist_name: String,
    pub artwork: Artwork,
    #[serde(default)]
    pub content_rating: ContentRating,
    #[serde(default)]
    pub disc_number: Option<u64>,
    pub duration_in_millis: u64,
    #[serde(default)]
    pub genre_names: Vec<String>,
    pub has_lyrics: bool,
    pub name: String,
    #[serde(default)]
    pub play_params: Option<PlayParams>,
    #[serde(default)]
    pub release_date: Option<NaiveDate>,
    #[serde(default)]
    pub track_number: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibrarySong {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: ObjectType,
    pub href: String,
    pub attributes: LibrarySongAttributes,
}

impl ToRequestObject for LibrarySong {
    fn id(&self) -> &str {
        &self.id
    }
    fn object_type(&self) -> &ObjectType {
        &self.object_type
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongAttributes {
    pub album_name: String,
    pub artist_name: String,
    #[serde(default)]
    pub artist_url: Option<String>,
    pub artwork: Artwork,
    #[serde(default)]
    pub attribution: Option<String>,
    #[serde(default)]
    pub audio_variants: Vec<AudioVariant>,
    #[serde(default)]
    pub composer_name: Option<String>,
    #[serde(default)]
    pub content_rating: ContentRating,
    #[serde(default)]
    pub disc_number: Option<u64>,
    pub duration_in_millis: u64,
    #[serde(default)]
    pub genre_names: Vec<String>,
    pub has_lyrics: bool,
    pub is_apple_digital_master: bool,
    #[serde(default)]
    pub isrc: Option<String>,
    #[serde(default)]
    pub movement_count: Option<u64>,
    #[serde(default)]
    pub movement_name: Option<String>,
    #[serde(default)]
    pub movement_number: Option<u64>,
    pub name: String,
    #[serde(default)]
    pub play_params: Option<PlayParams>,
    #[serde(default)]
    pub previews: Vec<Preview>,
    #[serde(default)]
    pub release_date: Option<NaiveDate>,
    #[serde(default)]
    pub track_number: Option<u64>,
    pub url: String,
    #[serde(default)]
    pub work_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Song {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: ObjectType,
    pub href: String,
    pub attributes: SongAttributes,
}

impl ToRequestObject for Song {
    fn id(&self) -> &str {
        &self.id
    }
    fn object_type(&self) -> &ObjectType {
        &self.object_type
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Description {
    pub standard: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistAttributes {
    #[serde(default)]
    pub artwork: Option<Artwork>,
    pub curator_name: String,
    #[serde(default)]
    pub description: Option<Description>,
    pub is_chart: bool,
    #[serde(default)]
    pub last_modified_date: Option<DateTime<Utc>>,
    pub name: String,
    pub playlist_type: PlaylistType,
    #[serde(default)]
    pub play_params: Option<PlayParams>,
    pub url: String,
    #[serde(default)]
    pub track_types: Vec<ObjectType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Playlist {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: ObjectType,
    pub href: String,
    pub attributes: PlaylistAttributes,
}

impl ToRequestObject for Playlist {
    fn id(&self) -> &str {
        &self.id
    }
    fn object_type(&self) -> &ObjectType {
        &self.object_type
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPlaylistAttributes {
    #[serde(default)]
    pub artwork: Option<Artwork>,
    pub can_edit: bool,
    #[serde(default)]
    pub date_added: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_modified_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub description: Option<Description>,
    pub has_catalog: bool,
    pub name: String,
    #[serde(default)]
    pub play_params: Option<PlayParams>,
    pub is_public: bool,
    #[serde(default)]
    pub track_types: Vec<ObjectType>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryPlaylist {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: ObjectType,
    pub href: String,
    pub attributes: LibraryPlaylistAttributes,
}

impl ToRequestObject for LibraryPlaylist {
    fn id(&self) -> &str {
        &self.id
    }
    fn object_type(&self) -> &ObjectType {
        &self.object_type
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult<T> {
    #[serde(default)]
    pub next: Option<String>,
    pub href: String,
    pub data: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SearchResults {
    // #[serde(default)]
    // pub activities: Option<SearchResult<Activity>>,
    // #[serde(default)]
    // pub albums: Option<SearchResult<Album>>,
    // #[serde(default)]
    // pub apple_curators: Option<SearchResult<AppleCurator>>,
    // #[serde(default)]
    // pub artists: Option<SearchResult<Artist>>,
    // #[serde(default)]
    // pub curators: Option<SearchResult<Curator>>,
    // #[serde(default)]
    // pub music_videos: Option<SearchResult<MusicVideo>>,
    #[serde(default)]
    pub playlists: Option<SearchResult<Playlist>>,
    // #[serde(default)]
    // pub record_labels: Option<SearchResult<RecordLabel>>,
    #[serde(default)]
    pub songs: Option<SearchResult<Song>>,
    // #[serde(default)]
    // pub stations: Option<SearchResult<Station>>,
    // #[serde(default)]
    // pub top: Option<SearchResult<Top>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultsMeta {
    pub order: Vec<String>,
    pub raw_order: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchResponseMeta {
    pub results: SearchResultsMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SuccessfulSearchResponse {
    pub results: SearchResults,
    pub meta: SearchResponseMeta,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SearchResponse {
    Success(SuccessfulSearchResponse),
    Error(ErrorResponse),
}