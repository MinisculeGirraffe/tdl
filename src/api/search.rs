use anyhow::Error;
use serde::de::DeserializeOwned;
use tabled::{Table, Tabled};

use super::get_items;
use super::API_BASE;
use tabled::TableIteratorExt;


pub async fn search_content<'a, T>(url: &str, query: &str, max: Option<u32>) -> Result<Table, Error>
where
    T: DeserializeOwned + 'a + Tabled,
{
    let url = format!("{}/search/{}", API_BASE, url);
    let query = ("query".to_string(), query.to_string());
    let table = get_items::<T>(&url, Some(vec![query]), max).await?.table();

    Ok(table)
}
