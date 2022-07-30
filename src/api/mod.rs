use crate::config::CONFIG;
use anyhow::Error;
use log::debug;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::de::DeserializeOwned;

use self::models::ItemResponse;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};

pub mod auth;
pub mod media;
pub mod models;
pub mod search;
static API_BASE: &str = "https://api.tidalhifi.com/v1";
static AUTH_BASE: &str = "https://auth.tidal.com/v1/oauth2";

// Share reqwest client for connection pooling
lazy_static::lazy_static! {
    pub static ref CLIENT: reqwest::Client = reqwest::Client::builder()
    //don't use the system openssl
    //use the example chrome useragent from MDN Docs as tidal API's will sometimes fail without it
    .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59")
    .build()
    .expect("Unable to build Reqwest Client");

    pub static ref REQ: ClientWithMiddleware  = ClientBuilder::new(CLIENT.clone())
    .with(
        RetryTransientMiddleware::new_with_policy(
         ExponentialBackoff {
            max_n_retries: 5,
            max_retry_interval: std::time::Duration::from_millis(1000),
            min_retry_interval: std::time::Duration::from_millis(2000),
            backoff_exponent: 2,
         })
    )
    .build();
}

async fn get<'a, T>(url: &'a str, query: &[(String, String)], auth: &'a String) -> Result<T, Error>
where
    T: DeserializeOwned + 'a,
{
    let req = REQ
        .get(url)
        .bearer_auth(auth)
        .query(&query)
        .send()
        .await?
        .text()
        .await?;

    debug!("{}", req);
    let result = serde_json::from_str::<T>(&req)?;

    Ok(result)
}

pub async fn get_items<'a, T>(
    url: &str,
    opts: Option<Vec<(String, String)>>,
    max: Option<u32>,
) -> Result<Vec<T>, Error>
where
    T: DeserializeOwned + 'a,
{
    let (token, country_code) = get_api_param().await?;
    let mut limit = 50;
    let mut offset = 0;
    let max = max.unwrap_or(u32::MAX);
    let mut params = vec![
        ("limit".to_string(), limit.to_string()),
        ("offset".to_string(), offset.to_string()),
        country_code,
    ];
    if let Some(opt) = opts {
        params.extend(opt);
    }

    let mut result: Vec<T> = Vec::new();
    'req: loop {
        let json = get::<ItemResponse<T>>(url, &params, &token).await?;
        limit = json.limit;
        // the minimum between the items in the response, and the total number of items requested
        let item_limit = u32::min(json.total_number_of_items, max);
        for item in json.items {
            if result.len() as u32 >= item_limit {
                break 'req;
            }
            result.push(item);
        }
        offset += limit;
    }
    Ok(result)
}

async fn get_api_param() -> Result<(String, (String, String)), Error> {
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

async fn get_country_code() -> Result<(String, String), Error> {
    let config = CONFIG.read().await;
    let country = config
        .login_key
        .country_code
        .clone()
        .ok_or_else(|| Error::msg("Missing Auth Token"))?;

    Ok((String::from("countryCode"), country))
}
