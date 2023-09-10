use serde::Serialize;

use super::ObjectType;

#[derive(Debug, Clone, Serialize)]
pub struct Object {
	pub id: String,
	#[serde(rename = "type")]
	pub object_type: ObjectType,
}

#[derive(Debug, Clone, Serialize)]
pub struct Objects {
	pub data: Vec<Object>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryPlaylistCreationRequestAttributes {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryPlaylistCreationRequestRelationships {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tracks: Option<Objects>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub parent: Option<Objects>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LibraryPlaylistCreationRequest {
	pub attributes: LibraryPlaylistCreationRequestAttributes,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub relationships: Option<LibraryPlaylistCreationRequestRelationships>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct SearchQuery<'a> {
	/// (Required) The entered text for the search with ‘+’ characters between each word, to replace spaces (for example term=james+br).
	pub term: &'a str,

	/// The localization to use, specified by a language tag. The possible values are in the supportedLanguageTags array belonging to the Storefront object specified by storefront. Otherwise, the default is defaultLanguageTag in Storefront.
	#[serde(rename = "l", skip_serializing_if = "Option::is_none")]
	pub localization: Option<&'a str>,

	/// The number of objects or number of objects in the specified relationship returned.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub limit: Option<usize>,

	/// The next page or group of objects to fetch.
	///
	/// - Default: 5
	/// - Maximum Value: 25
	#[serde(skip_serializing_if = "Option::is_none")]
	pub offset: Option<usize>,

	/// (Required) The (comma-separated) list of the types of resources to include in the results.
	///
	/// Possible Values: activities, albums, apple-curators, artists, curators, music-videos, playlists, record-labels, songs, stations
	pub types: &'a str,

	/// A (comma-separated) list of modifications to apply to the request.
	///
	/// - Value: topResults
	#[serde(skip_serializing_if = "Option::is_none")]
	pub with: Option<&'a str>,
}