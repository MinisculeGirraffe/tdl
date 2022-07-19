use std::collections::HashMap;

use anyhow::Error;

use crate::api::AUTH_BASE;
use crate::config::CONFIG;

use super::{get_auth_token, models::*};
use super::{API_BASE, REQ};

pub async fn get_device_code() -> Result<DeviceAuthResponse, Error> {
    let config = CONFIG.read().await;
    let client_id = config.api_key.client_id.clone();
    let data = DeviceAuthRequest {
        client_id: client_id.clone(),
        scope: Some("r_usr+w_usr+w_sub".to_string()),
        ..Default::default()
    };
    let body = serde_urlencoded::to_string(&data)?;
    let req = REQ
        .post(format!("{}/device_authorization", &AUTH_BASE))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    if !req.status().is_success() {
        return Err(Error::msg("Failed to get device code"));
    }

    let device_key = req.json::<DeviceAuthResponse>().await?;
    Ok(device_key)
}

pub async fn verify_access_token(access_token: &str) -> Result<bool, Error> {
    let req = REQ
        .get(format!("{}/sessions", &API_BASE))
        .bearer_auth(access_token)
        .send()
        .await?;
    Ok(req.status().is_success())
}

pub async fn _login_access_token(access_token: &str, user_id: Option<&str>) -> Result<(), Error> {
    let req = REQ
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

    {
        let mut config = CONFIG.write().await;
        //config.login_key.user_id = Some(req.get("userId"));
        config.login_key.country_code = Some(req.get("countryCode").unwrap().to_string());
        config.login_key.access_token = Some(access_token.to_string());
        config.save()?;
    }

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

    let req = REQ
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
    let req = REQ
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
            return Err(Error::msg("Failed to check auth status"));
        }
    }
    let res = req.json::<RefreshResponse>().await?;
    Ok(res)
}

pub async fn logout() -> Result<(), Error> {
    let token = get_auth_token().await?;
    let _ = REQ
        .post("https://api.tidal.com/v1/logout")
        .bearer_auth(token)
        .send()
        .await?;

    {
        let mut config = CONFIG.write().await;
        config.login_key.access_token = None;
        config.login_key.refresh_token = None;
        config.save()?;
    }
    Ok(())
}
