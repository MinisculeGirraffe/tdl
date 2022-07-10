use std::collections::HashMap;

use anyhow::Error;

use serde::{Deserialize, Serialize};

use crate::config::CONFIG;

#[derive(Debug, Serialize, Deserialize)]
pub enum AudioQuality {
    Normal = 0,
    High = 1,
    HiFi = 2,
    Master = 3,
}

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

pub async fn get_device_code() -> Result<DeviceAuthResponse, Error> {
    let config = CONFIG.read().await;
    println!("Getting device code...");
    let client_id = config.api_key.client_id.clone();
    let data = DeviceAuthRequest {
        client_id: client_id.clone(),
        scope: Some("r_usr+w_usr+w_sub".to_string()),
        ..Default::default()
    };
    let body = serde_urlencoded::to_string(&data)?;

    let req = reqwest::Client::new()
        .post("https://auth.tidal.com/v1/oauth2/device_authorization")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    if !req.status().is_success() {
        return Err(Error::msg("Failed to get device code"));
    }

    let device_key = req.json::<DeviceAuthResponse>().await?;
    println!("Got device code: {:?}", device_key);
    Ok(device_key)
}

pub async fn verify_access_token(access_token: &str) -> Result<bool, Error> {
    let req = reqwest::Client::new()
        .get("https://api.tidal.com/v1/sessions")
        .bearer_auth(access_token)
        .send()
        .await?;
    Ok(req.status().is_success())
}

pub async fn login_access_token(access_token: &str, user_id: Option<&str>) -> Result<(), Error> {
    let req = reqwest::Client::new()
        .get("https://api.tidal.com/v1/sessions")
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
    println!("got config lock");
    //config.login_key.user_id = Some(req.get("userId"));
    config.login_key.country_code = Some(req.get("countryCode").unwrap().to_string());
    config.login_key.access_token = Some(access_token.to_string());
    config.save()?;
    Ok(())
}
pub async fn refresh_access_token(refresh_token: &str) -> Result<(), Error> {
    let mut config = CONFIG.write().await;
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
        let now = chrono::Utc::now().timestamp();
        config.login_key.user_id = Some(res.user.user_id);
        config.login_key.country_code = Some(res.user.country_code);
        config.login_key.access_token = Some(res.access_token);
        config.login_key.refresh_token = Some(res.refresh_token);
        config.login_key.expires_after = Some(now + res.expires_in);
        config.save()?;
        Ok(())
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
        .post("https://auth.tidal.com/v1/oauth2/token")
        .basic_auth(client_id, Some(client_secret))
        .body(body)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await?;
    if !req.status().is_success() {
        if req.status().is_client_error() {
            return Err(Error::msg(req.status().canonical_reason().unwrap()));
        } else {
            println!("{:?}", req.status());
            return Err(Error::msg("Failed to check auth status"));
        }
    }
    let res = req.json::<RefreshResponse>().await?;
    println!("Got refresh response: {:?}", res);

    Ok(res)
}
