use super::{get_api_param, models::*};
use super::{API_BASE, REQ};
use crate::config::CONFIG;
use anyhow::Error;
use serde::de::DeserializeOwned;
use std::str::FromStr;

pub async fn get_track(id: usize) -> Result<Track, Error> {
    let (token, country_code) = get_api_param().await?;
    let url = format!("{}/tracks/{}", API_BASE, id);

    let res = REQ
        .get(url)
        .bearer_auth(token)
        .query(&[("countryCode", country_code)])
        .send()
        .await?
        .json::<Track>()
        .await?;

    Ok(res)
}

pub async fn get_items<'a, T>(url: &str, opts: Option<Vec<(&str, String)>>) -> Result<Vec<T>, Error>
where
    T: DeserializeOwned + 'a,
{
    let (token, country_code) = get_api_param().await?;
    let limit = 50;
    let mut offset = 0;

    let mut params = vec![
        ("limit", limit.to_string()),
        ("offset", offset.to_string()),
        ("countryCode", country_code),
    ];
    if let Some(opt) = opts {
        params.extend(opt);
    }

    let mut result: Vec<T> = Vec::new();
    loop {
        let json = REQ
            .get(url)
            .query(&params)
            .bearer_auth(&token)
            .send()
            .await?
            .json::<ItemResponse<T>>()
            .await?;

        let length = json.items.len();
        for item in json.items {
            result.push(item);
        }
        if length < limit {
            break;
        }
        offset += limit;
    }
    Ok(result)
}

pub async fn get_album(id: usize) -> Result<Album, Error> {
    let (token, country_code) = get_api_param().await?;
    let url = format!("{}/albums/{}", API_BASE, id);

    let res = REQ
        .get(url)
        .bearer_auth(token)
        .query(&[("countryCode", country_code)])
        .send()
        .await?
        .json::<Album>()
        .await?;
    Ok(res)
}

pub async fn get_stream_url(id: usize) -> Result<PlaybackManifest, Error> {
    let config = CONFIG.read().await;
    let (token, country_code) = get_api_param().await?;
    let url = format!("{}/tracks/{}/playbackinfopostpaywall", &API_BASE, id);
    let query = &[
        ("countryCode", &country_code),
        ("audioquality", &config.audio_quality.to_string()),
        ("playbackmode", &PlaybackMode::Stream.to_string()),
        ("assetpresentation", &AssetPresentation::Full.to_string()),
    ];
    let req = REQ
        .get(url)
        .query(query)
        .bearer_auth(token)
        .send()
        .await?
        .json::<PlaybackInfoPostPaywallRes>()
        .await?;

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
    let req = reqwest::get(get_cover_url(id, 1280, 1280)).await?;
    let content_type = req
        .headers()
        .get("Content-Type")
        .unwrap()
        .to_str()?
        .to_string();
    let data = req.bytes().await?.to_vec();

    Ok(Cover { content_type, data })
}
