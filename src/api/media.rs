use super::{get, get_api_param, models::*};
use super::{API_BASE, REQ};
use crate::config::CONFIG;
use anyhow::Error;

use std::str::FromStr;

pub async fn get_track(id: usize) -> Result<Track, Error> {
    let (token, country_code) = get_api_param().await?;
    let url = format!("{}/tracks/{}", API_BASE, id);

    let res = get::<Track>(&url, &[country_code], &token).await?;

    Ok(res)
}

pub async fn get_album(id: usize) -> Result<Album, Error> {
    let (token, country_code) = get_api_param().await?;
    let url = format!("{}/albums/{}", API_BASE, id);

    let res = get::<Album>(&url, &[country_code], &token).await?;

    Ok(res)
}

pub async fn get_stream_url(id: usize) -> Result<PlaybackManifest, Error> {
    let config = CONFIG.read().await;
    let (token, country_code) = get_api_param().await?;
    let url = format!("{}/tracks/{}/playbackinfopostpaywall", &API_BASE, id);
    let query = &[
        country_code,
        ("audioquality".to_string(), config.audio_quality.to_string()),
        ("playbackmode".to_string(), PlaybackMode::Stream.to_string()),
        (
            "assetpresentation".to_string(),
            AssetPresentation::Full.to_string(),
        ),
    ];

    let req = get::<PlaybackInfoPostPaywallRes>(&url, query, &token).await?;

    match req.manifest_mime_type.as_str() {
        "application/vnd.tidal.bts" => Ok(PlaybackManifest::from_str(&req.manifest)?),
        _ => Err(Error::msg("Incorrect Mimetype on Response")),
    }
}

pub fn get_cover_url(id: &str, width: usize, height: usize) -> String {
    format!(
        "https://resources.tidal.com/images/{}/{}x{}.jpg",
        id.replace('-', "/"),
        width,
        height
    )
}

pub async fn get_cover_data(id: &str) -> Result<Cover, Error> {
    let req = REQ.get(get_cover_url(id, 1280, 1280)).send().await?;
    let content_type = req
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()?
        .to_string();
    let data = req.bytes().await?.to_vec();

    Ok(Cover { content_type, data })
}
