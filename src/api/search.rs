use anyhow::Error;

use super::get_items;
use super::models::{Album, Artist, Track};
use super::API_BASE;

pub async fn _search_all(_query: &str) {}

pub async fn search_artist(query: String, max: Option<u32>) -> Result<Vec<Artist>, Error> {
    let url = format!("{}/search/artists", API_BASE);
    let query = ("query".to_string(), query);
    get_items::<Artist>(&url, Some(vec![query]), max).await
}
pub async fn search_track(query: String, max: Option<u32>) -> Result<Vec<Track>, Error> {
    let url = format!("{}/search/tracks", API_BASE);
    let query = ("query".to_string(), query);
    get_items::<Track>(&url, Some(vec![query]), max).await
}

pub async fn search_album(query: String, max: Option<u32>) -> Result<Vec<Album>, Error> {
    let url = format!("{}/search/albums", API_BASE);
    let query = ("query".to_string(), query);
    get_items::<Album>(&url, Some(vec![query]), max).await
}
