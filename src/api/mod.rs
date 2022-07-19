use anyhow::Error;

use crate::config::CONFIG;

pub mod auth;
pub mod media;
pub mod models;

static API_BASE: &str = "https://api.tidalhifi.com/v1";
static AUTH_BASE: &str = "https://auth.tidal.com/v1/oauth2";

// Share reqwest client for connection pooling
lazy_static::lazy_static! {
     static ref REQ: reqwest::Client  = reqwest::Client::builder()
    //don't use the system openssl
    .use_rustls_tls()
    //use the example chrome useragent from MDN Docs as tidal API's will sometimes fail without it
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59")
    .build()
    .unwrap();
}

async fn get_api_param() -> Result<(String, String), Error> {
    Ok((get_auth_token().await?, get_country_code().await?))
}

async fn get_auth_token() -> Result<String, Error> {
    let config = CONFIG.read().await;
    config
        .login_key
        .access_token
        .clone()
        .ok_or_else(|| Error::msg("Missing Auth Token"))
}

async fn get_country_code() -> Result<String, Error> {
    let config = CONFIG.read().await;
    config
        .login_key
        .country_code
        .clone()
        .ok_or_else(|| Error::msg("Missing Auth Token"))
}
