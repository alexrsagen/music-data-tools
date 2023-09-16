mod apple_music;
mod args;
mod config;
mod logger;
mod my_spotify_data;

use apple_music::request::{
    LibraryPlaylistCreationRequest, LibraryPlaylistCreationRequestAttributes,
    LibraryPlaylistCreationRequestRelationships, Objects, SearchQuery,
};
use apple_music::response::{ListResponse, SearchResponse, PaginatedResponse};
use apple_music::{ObjectType, ToRequestObject};
use my_spotify_data::{
    Playlist as SpotifyPlaylist, PlaylistItemAbstraction, Playlists as SpotifyPlaylists,
};
use strsim::normalized_damerau_levenshtein;

use std::path::Path;

use anyhow::Result;
use console::Term;
use dialoguer::theme::ColorfulTheme;

#[tokio::main]
async fn main() -> Result<()> {
    // parse command-line arguments
    let args = args::Args::parse();

    // set up logger
    logger::try_init(args.log_level)?;

    // load or create default config
    let config = config::Config::load_or_init(&args.config_path)?;

    match args.command {
        args::Command::ImportSpotifyGdprPlaylistsToAppleMusicApi {
            playlist_file,
            playlists,
            dry,
            min_score,
            limit,
        } => import_spotify_playlists_to_apple_music(&config, playlist_file, playlists, dry, min_score, limit).await?,
    };

    Ok(())
}

fn select_spotify_playlists<'a>(
    term: &Term,
    playlists: &'a [SpotifyPlaylist],
    selected_names: &[String],
) -> Result<Vec<&'a SpotifyPlaylist>> {
    if dialoguer::console::user_attended() {
        let playlists_checked: Vec<(&SpotifyPlaylist, bool)> = playlists
            .iter()
            .map(|p| (p, selected_names.contains(&p.name)))
            .collect();

        let selected_playlists_indices =
            dialoguer::MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt("Select Spotify playlists to import to Apple Music")
                .items_checked(&playlists_checked)
                .max_length(10)
                .interact_on(term)?;

        Ok(playlists
            .iter()
            .enumerate()
            .filter(|(i, _)| selected_playlists_indices.contains(i))
            .map(|(_, p)| p)
            .collect())
    } else {
        Ok(playlists
            .iter()
            .filter(|p| selected_names.contains(&p.name))
            .collect())
    }
}

async fn import_spotify_playlists_to_apple_music<P: AsRef<Path>>(
    config: &config::Config,
    playlist_file: P,
    playlists: Option<Vec<String>>,
    dry: bool,
    min_score: f64,
    limit: usize,
) -> anyhow::Result<()> {
    let playlists = playlists.unwrap_or_default();
    let term = console::Term::stdout();
    let music_client = apple_music::Client::new(&config.apple_music_user_token);

    term.write_str(&format!(
        "{} Loading playlists from Spotify data export... ",
        console::style("✱").blue()
    ))?;

    let spotify_playlists = tokio::fs::read_to_string(playlist_file).await?;
    let spotify_playlists: SpotifyPlaylists = serde_json::from_str(&spotify_playlists)?;

    term.clear_line()?;
    term.write_line(&format!(
        "{} Loaded {} playlist{} from Spotify data export",
        console::style("✔").green(),
        spotify_playlists.playlists.len(),
        spotify_playlists
            .playlists
            .len()
            .gt(&1)
            .then_some("s")
            .unwrap_or_default()
    ))?;

    let selected_spotify_playlists =
        select_spotify_playlists(&term, &spotify_playlists.playlists, &playlists)?;

    term.write_str(&format!(
        "{} Loading playlists from Apple Music... ",
        console::style("✱").blue()
    ))?;

    let music_playlists = music_client
        .get_library_playlists()
        .await?
        .all(&music_client)
        .await?;

    for playlist in &music_playlists {
        log::debug!("Playlist {}: {}", playlist.attributes.name, playlist.id);
        music_client
            .get_library_playlist_tracks(&playlist.id)
            .await?;
    }

    term.clear_line()?;
    term.write_line(&format!(
        "{} Loaded {} playlist{} from Apple Music",
        console::style("✔").green(),
        music_playlists.len(),
        music_playlists
            .len()
            .gt(&1)
            .then_some("s")
            .unwrap_or_default()
    ))?;

    for playlist in &selected_spotify_playlists {
        if !dry && music_playlists
            .iter()
            .find(|p| p.attributes.name == playlist.name)
            .is_some()
        {
            term.write_line(&format!(
                "{} Playlist {} already exists in Apple Music",
                console::style("✔").green(),
                playlist.name
            ))?;
            continue;
        }

        term.write_line(&format!(
            "{} Processing playlist {:?}",
            console::style("✱").blue(),
            playlist.name,
        ))?;

        // [Object { id: String::from("a.1247588952"), object_type: apple_music::ObjectType::LibrarySongs }]
        let mut music_track_objects = Vec::new();

        for item in &playlist.items {
            if let PlaylistItemAbstraction::Track(track) = item.item() {
                let search_term = format!("{} {}", track.artist_name, track.track_name);

                term.write_str(&format!(
                    "\t{} Searching for {:?} in the Apple Music catalog... ",
                    console::style("✱").blue(),
                    search_term
                ))?;

                let search_res = music_client
                    .search_catalog(
                        &config.apple_music_storefront,
                        &SearchQuery {
                            term: &search_term,
                            types: ObjectType::Songs.as_str(),
                            limit: Some(limit),
                            ..Default::default()
                        },
                    )
                    .await?;

                if let Some(songs) = &search_res.results.songs {
                    // score songs by fuzzy match of artist, album and track name
                    let mut songs_with_score = Vec::with_capacity(songs.data.len());
                    for song in &songs.data {
                        let artist_score = normalized_damerau_levenshtein(&track.artist_name, &song.attributes.artist_name);
                        let album_score = normalized_damerau_levenshtein(&track.album_name, &song.attributes.album_name);
                        let track_score = normalized_damerau_levenshtein(&track.track_name, &song.attributes.name);

                        let compound_score = artist_score + album_score + track_score;
                        if compound_score > min_score {
                            songs_with_score.push((song, compound_score));
                        }
                    }
                    songs_with_score.sort_by(|(_, a_score), (_, b_score)| b_score.total_cmp(a_score));

                    if let Some((song, score)) = songs_with_score.first() {
                        music_track_objects.push(song.to_request_object());

                        term.clear_line()?;
                        term.write_line(&format!(
                            "\t{} Found \"{} - {}\" in the Apple Music catalog (score: {:.6}): {}",
                            console::style("✔").green(),
                            &song.attributes.artist_name,
                            &song.attributes.name,
                            score,
                            &song.attributes.url
                        ))?;

                        continue;
                    }
                }

                term.clear_line()?;
                term.write_line(&format!(
                    "\t{} Skipping {:?}: Could not be found in the Apple Music catalog",
                    console::style("✘").red(),
                    &search_term
                ))?;
            } else {
                term.clear_line()?;
                term.write_line(&format!(
                    "\t{} Skipping {:?}: Not a track",
                    console::style("✘").red(),
                    &item
                ))?;
            }
        }

        if dry {
            term.write_str(&format!(
                "{} Skipped creating playlist {:?} in Apple Music (--dry)",
                console::style("✱").blue(),
                &playlist.name
            ))?;

            continue;
        }

        let create_playlist_res = music_client
            .create_library_playlist(&LibraryPlaylistCreationRequest {
                attributes: LibraryPlaylistCreationRequestAttributes {
                    name: playlist.name.clone(),
                    description: playlist.description.clone(),
                },
                relationships: Some(LibraryPlaylistCreationRequestRelationships {
                    tracks: Some(Objects {
                        data: music_track_objects,
                    }),
                    parent: None,
                }),
            })
            .await;

        match create_playlist_res {
            Ok(res) => term.write_line(&format!(
                "{} Created playlist {:?} in Apple Music: {}",
                console::style("✔").green(),
                &playlist.name,
                res
                    .data
                    .first()
                    .map(|p| format!(
                        "https://music.apple.com/{}/library/playlist/{}",
                        &config.apple_music_storefront, &p.id
                    ))
                    .unwrap_or_default()
            ))?,

            Err(e) => term.write_line(&format!(
                "{} Failed to create playlist {:?} in Apple Music: {}",
                console::style("✘").red(),
                &playlist.name,
                e
            ))?,
        }
    }

    Ok(())
}
