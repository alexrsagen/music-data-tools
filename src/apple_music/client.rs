use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Result};
use reqwest::Url;
use select::document::Document;
use select::predicate::Name;
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use tokio::sync::OnceCell;

use super::request::{LibraryPlaylistCreationRequest, Objects, SearchQuery};
use super::response::{ListResponse, LibrarySong, LibraryPlaylist, SearchResponse};

fn char_windows<'a>(src: &'a str, win_size: usize) -> impl Iterator<Item = (usize, &'a str)> {
    src.char_indices().flat_map(move |(from, _)| {
        src[from..]
            .char_indices()
            .skip(win_size - 1)
            .next()
            .map(|(to, c)| (from, &src[from..from + to + c.len_utf8()]))
    })
}

fn str_byteindex_of_substr(haystack: &str, needle: &str) -> Option<usize> {
    char_windows(haystack, needle.len()).find_map(|(pos, window)| (window == needle).then_some(pos))
}

fn str_byteindex_of_predicate<P>(haystack: &str, predicate: P) -> Option<usize>
where
    P: Fn(char) -> bool,
{
    haystack
        .char_indices()
        .find_map(|(pos, c)| predicate(c).then_some(pos))
}

async fn get_token() -> Result<String> {
    // get HTML
    let base_url = Url::from_str("https://music.apple.com")?;
    let html = reqwest::get(base_url.clone()).await?.text().await?;
    let doc = Document::from(html.as_str());

    // get JS URL from HTML
    let js_relative_url = doc
        .find(Name("script"))
        .filter_map(|script| {
            script
                .attr("src")
                .and_then(|src| src.starts_with("/assets/index-").then_some(src))
        })
        .next()
        .ok_or(anyhow!("could not find JS in HTML"))?;
    let js_url = base_url.join(js_relative_url)?;

    // get JS
    let js = reqwest::get(js_url).await?.text().await?;

    // get position of JWT from JS
    let start = str_byteindex_of_substr(
        &js,
        "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6IldlYlBsYXlLaWQifQ.",
    )
    .ok_or(anyhow!("could not find start of token in JS"))?;
    let len = str_byteindex_of_predicate(&js[start..], |c| c == '"' || c == '\'')
        .ok_or(anyhow!("could not find end of token in JS"))?;
    let end = start + len;

    // return JWT
    Ok(js[start..end].to_string())
}

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub user_token: String,
    pub max_retries: usize,
    pub retry_interval: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            user_token: String::new(),
            max_retries: 30,
            retry_interval: Duration::from_secs(1),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    config: Arc<ClientConfig>,
    token: OnceCell<String>,
    client: reqwest::Client,
}

impl Client {
    async fn get_token(&self) -> Result<&str> {
        self.token
            .get_or_try_init(get_token)
            .await
            .map(|s| s.as_str())
    }

    pub fn new(user_token: &str) -> Self {
        Self::new_with_config(ClientConfig {
            user_token: user_token.to_string(),
            ..Default::default()
        })
    }

    pub fn new_with_config(config: ClientConfig) -> Self {
        Self {
            config: Arc::new(config),
            token: OnceCell::new(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_with_query<Q: Serialize + ?Sized, O: DeserializeOwned>(&self, endpoint: &str, query: Option<&Q>) -> Result<O> {
        let base_url = Url::from_str("https://api.music.apple.com")?;
        let url = base_url.join(endpoint)?;

        let apple_music_token = self.get_token().await?;

        let mut text = String::new();
        for retry in 0..self.config.max_retries {
            if retry > 0 {
                tokio::time::sleep(self.config.retry_interval).await;
            }

            log::debug!("Apple Music: GET {}", &url);
            let mut builder = self.client.get(url.clone())
                .header("Music-User-Token", &self.config.user_token)
                .header("Origin", "https://music.apple.com")
                .bearer_auth(&apple_music_token);

            if let Some(query) = query {
                builder = builder.query(query);
            }

            let res = builder.send().await?;
            let status = res.status();
            let version = res.version();
            text = res.text().await?;
            log::debug!("Apple Music: {:?} {}: {}", version, status, text);

            if !status.is_server_error() {
                break;
            }
        }

        Ok(serde_json::from_str(&text)?)
    }

    pub async fn get<O: DeserializeOwned>(&self, endpoint: &str) -> Result<O> {
        self.get_with_query::<(), O>(endpoint, None).await
    }

    pub async fn post<I: Serialize + ?Sized, O: DeserializeOwned>(&self, endpoint: &str, data: &I) -> Result<O> {
        let base_url = Url::from_str("https://api.music.apple.com")?;
        let url = base_url.join(endpoint)?;

        let apple_music_token = self.get_token().await?;

        let mut text = String::new();
        for retry in 0..self.config.max_retries {
            if retry > 0 {
                tokio::time::sleep(self.config.retry_interval).await;
            }

            log::debug!("Apple Music: POST {}", &url);
            let res = self.client
                .post(url.clone())
                .json(data)
                .header("Music-User-Token", &self.config.user_token)
                .header("Origin", "https://music.apple.com")
                .bearer_auth(&apple_music_token)
                .send()
                .await?;

            let status = res.status();
            let version = res.version();
            text = res.text().await?;
            log::debug!("Apple Music: {:?} {}: {}", version, status, text);

            if !status.is_server_error() {
                break;
            }
        }

        Ok(serde_json::from_str(&text)?)
    }

    #[allow(unused)]
    pub async fn post_no_content<I: Serialize + ?Sized>(&self, endpoint: &str, data: &I) -> Result<()> {
        let base_url = Url::from_str("https://api.music.apple.com")?;
        let url = base_url.join(endpoint)?;

        let apple_music_token = self.get_token().await?;

        for retry in 0..self.config.max_retries {
            if retry > 0 {
                tokio::time::sleep(self.config.retry_interval).await;
            }

            log::debug!("Apple Music: POST {}", &url);
            let res = self.client
                .post(url.clone())
                .json(data)
                .header("Music-User-Token", &self.config.user_token)
                .header("Origin", "https://music.apple.com")
                .bearer_auth(&apple_music_token)
                .send()
                .await?;

            let status = res.status();
            let version = res.version();
            log::debug!("Apple Music: {:?} {}", version, status);

            if !status.is_server_error() {
                break;
            }
        }

        Ok(())
    }

    #[allow(unused)]
    pub async fn get_library_songs(&self) -> Result<ListResponse<LibrarySong>> {
        self.get("/v1/me/library/songs").await
    }

    pub async fn get_library_playlists(&self) -> Result<ListResponse<LibraryPlaylist>> {
        self.get("/v1/me/library/playlists").await
    }

    pub async fn create_library_playlist(&self, data: &LibraryPlaylistCreationRequest) -> Result<ListResponse<LibraryPlaylist>> {
        self.post("/v1/me/library/playlists", data).await
    }

    pub async fn get_library_playlist_tracks(&self, playlist_id: &str) -> Result<ListResponse<LibrarySong>> {
        self.get(&format!("/v1/me/library/playlists/{}/tracks", playlist_id)).await
    }

    #[allow(unused)]
    pub async fn add_library_playlist_tracks(&self, playlist_id: &str, tracks: &Objects) -> Result<()> {
        self.post_no_content(&format!("/v1/me/library/playlists/{}/tracks", playlist_id), tracks).await
    }

    pub async fn search_catalog(&self, storefront: &str, query: &SearchQuery<'_>) -> Result<SearchResponse> {
        self.get_with_query(&format!("/v1/catalog/{}/search", storefront), Some(query)).await
    }
}
