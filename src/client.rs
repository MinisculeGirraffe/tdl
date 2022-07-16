use crate::{
    config::CONFIG,
    models::{Album, AssetPresentation, AudioQuality, PlaybackManifest, PlaybackMode, Track},
};
use anyhow::Error;
use log::{debug, info};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[derive(Serialize, Deserialize, Debug)]
pub struct RefreshResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct UserResponse {
    pub user_id: i64,
    pub country_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct DeviceAuthResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: i64,
    pub interval: i64,
}

#[derive(Serialize, Deserialize, Debug)]
struct DeviceAuthRequest {
    client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grant_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    device_code: Option<String>,
}

impl Default for DeviceAuthRequest {
    fn default() -> Self {
        DeviceAuthRequest {
            client_id: "".to_string(),
            client_secret: None,
            refresh_token: None,
            scope: None,
            grant_type: None,
            device_code: None,
        }
    }
}

static API_BASE: &str = "https://api.tidalhifi.com/v1";
static AUTH_BASE: &str = "https://auth.tidal.com/v1/oauth2";

pub async fn get_device_code() -> Result<DeviceAuthResponse, Error> {
    let config = CONFIG.read().await;
    info!("Getting device code...");
    let client_id = config.api_key.client_id.clone();
    let data = DeviceAuthRequest {
        client_id: client_id.clone(),
        scope: Some("r_usr+w_usr+w_sub".to_string()),
        ..Default::default()
    };
    let body = serde_urlencoded::to_string(&data)?;

    let req = reqwest::Client::new()
        .post(format!("{}/device_authorization", &AUTH_BASE))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    if !req.status().is_success() {
        return Err(Error::msg("Failed to get device code"));
    }

    let device_key = req.json::<DeviceAuthResponse>().await?;
    info!("Got device code: {:?}", device_key);
    Ok(device_key)
}

pub async fn verify_access_token(access_token: &str) -> Result<bool, Error> {
    let req = reqwest::Client::new()
        .get(format!("{}/sessions", &API_BASE))
        .bearer_auth(access_token)
        .send()
        .await?;
    Ok(req.status().is_success())
}

pub async fn _login_access_token(access_token: &str, user_id: Option<&str>) -> Result<(), Error> {
    let req = reqwest::Client::new()
        .get(format!("{}/sessions", &API_BASE))
        .bearer_auth(access_token)
        .send()
        .await?
        .json::<HashMap<String, String>>()
        .await?;

    if let Some(uid) = user_id {
        if let Some(ruid) = req.get("userId") {
            if ruid != uid {
                return Err(Error::msg("User ID mismatch"));
            }
        } else {
            return Err(Error::msg("User ID missing"));
        }
    }
    let mut config = CONFIG.write().await;
    debug!("got config lock");
    //config.login_key.user_id = Some(req.get("userId"));
    config.login_key.country_code = Some(req.get("countryCode").unwrap().to_string());
    config.login_key.access_token = Some(access_token.to_string());
    config.save()?;
    Ok(())
}
pub async fn refresh_access_token(refresh_token: &str) -> Result<RefreshResponse, Error> {
    let config = CONFIG.read().await;
    let client_id = &config.api_key.client_id;
    let client_secret = &config.api_key.client_secret;

    let data = DeviceAuthRequest {
        client_id: client_id.clone(),
        client_secret: Some(client_secret.clone()),
        refresh_token: Some(refresh_token.to_string()),
        grant_type: Some("refresh_token".to_string()),
        ..Default::default()
    };
    let body = serde_urlencoded::to_string(&data)?;

    let req = reqwest::Client::new()
        .post("https://auth.tidal.com/v1/oauth2/token")
        .body(body)
        .basic_auth(client_id, Some(client_secret))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?;
    if req.status().is_success() {
        let res = req.json::<RefreshResponse>().await?;
        Ok(res)
    } else {
        Err(Error::msg("Failed to refresh access token"))
    }
}

pub async fn check_auth_status(device_code: &str) -> Result<RefreshResponse, Error> {
    let config = &CONFIG.read().await;
    let client_id = &config.api_key.client_id;
    let client_secret = &config.api_key.client_secret;

    let data = DeviceAuthRequest {
        client_id: client_id.clone(),
        device_code: Some(device_code.to_string()),
        scope: Some("r_usr+w_usr+w_sub".to_string()),
        grant_type: Some("urn:ietf:params:oauth:grant-type:device_code".to_string()),
        ..Default::default()
    };
    let body = serde_urlencoded::to_string(&data)?;
    let req = reqwest::Client::new()
        .post(format!("{}/token", &AUTH_BASE))
        .basic_auth(client_id, Some(client_secret))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?;
    if !req.status().is_success() {
        if req.status().is_client_error() {
            return Err(Error::msg(req.status().canonical_reason().unwrap()));
        } else {
            debug!("{:?}", req.status());
            return Err(Error::msg("Failed to check auth status"));
        }
    }
    let res = req.json::<RefreshResponse>().await?;
    info!("Got refresh response: {:?}", res);

    Ok(res)
}

pub async fn get_track(id: usize) -> Result<Track, Error> {
    let config = CONFIG.read().await;
    let token = config.login_key.access_token.as_ref().unwrap();
    let url = format!("{}/tracks/{}", API_BASE, id);

    let res = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .query(&[(
            "countryCode",
            config.login_key.country_code.as_ref().unwrap(),
        )])
        .send()
        .await?
        .json::<Track>()
        .await?;

    Ok(res)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct ItemResponse<T> {
    limit: usize,
    offset: usize,
    total_number_of_items: usize,
    items: Vec<T>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ItemResponseItem<T> {
    pub item: T,
    #[serde(alias = "type")]
    pub item_type: String,
}
pub async fn get_items<'a, T>(url: &str, opts: Option<Vec<(&str, String)>>) -> Result<Vec<T>, Error>
where
    T: DeserializeOwned + 'a,
{
    let config = CONFIG.read().await;
    let limit = 50;
    let mut offset = 0;

    let mut params = vec![
        ("limit", limit.to_string()),
        ("offset", offset.to_string()),
        (
            "countryCode",
            config.login_key.country_code.as_ref().unwrap().to_owned(),
        ),
    ];
    if let Some(opt) = opts {
        params.extend(opt);
    }

    let mut result: Vec<T> = Vec::new();
    loop {
        let body = reqwest::Client::new()
            .get(url)
            .query(&params)
            .bearer_auth(config.login_key.access_token.as_ref().unwrap())
            .send()
            .await?
            .text()
            .await?;

        debug!("{}", &body);

        let json = serde_json::from_str::<ItemResponse<T>>(&body)?;

        let length = json.items.len();
        for item in json.items {
            result.push(item);
        }
        if length < 50 {
            break;
        }
        offset += 50;
    }
    Ok(result)
}

pub async fn get_album(id: usize) -> Result<Album, Error> {
    let config = CONFIG.read().await;
    let token = config.login_key.access_token.as_ref().unwrap();
    let country_code = config.login_key.country_code.as_ref().unwrap();
    let url = format!("{}/albums/{}", API_BASE, id);

    let res = reqwest::Client::new()
        .get(url)
        .bearer_auth(token)
        .query(&[("countryCode", country_code)])
        .send()
        .await?
        .json::<Album>()
        .await?;
    Ok(res)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct PlaybackInfoPostPaywallRes {
    track_id: usize,
    asset_presentation: AssetPresentation,
    audio_quality: AudioQuality,
    manifest_mime_type: String,
    manifest: String,
}

pub async fn get_stream_url(id: usize) -> Result<PlaybackManifest, Error> {
    let config = CONFIG.read().await;

    let url = format!("{}/tracks/{}/playbackinfopostpaywall", &API_BASE, id);
    let query = &[
        (
            "countryCode",
            config.login_key.country_code.as_ref().unwrap(),
        ),
        ("audioquality", &config.audio_quality.to_string()),
        ("playbackmode", &PlaybackMode::Stream.to_string()),
        ("assetpresentation", &AssetPresentation::Full.to_string()),
    ];
    let req = reqwest::Client::new()
        .get(url)
        .query(query)
        .bearer_auth(config.login_key.access_token.as_ref().unwrap())
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
pub struct Cover {
    pub content_type: String,
    pub data: Vec<u8>,
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
