use super::ApiClient;
use anyhow::Error;
use serde::de::DeserializeOwned;
use std::{ops::Deref, sync::Arc};
use tabled::{Table, TableIteratorExt, Tabled};

pub struct SearchClient(Arc<ApiClient>);

impl SearchClient {
    pub fn new(client: Arc<ApiClient>) -> Self {
        Self(client)
    }
}

impl Deref for SearchClient {
    type Target = ApiClient;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SearchClient {
    pub async fn search_content<'a, T>(
        &self,
        url: &str,
        query: &str,
        max: Option<usize>,
    ) -> Result<Table, Error>
    where
        T: DeserializeOwned + 'a + Tabled,
    {
        let url = format!("{}/search/{}", self.api_base, url);
        let query = ("query".to_string(), query.to_string());
        let table = self
            .get_items::<T>(&url, Some(vec![query]), max)
            .await?
            .table();

        Ok(table)
    }
}
