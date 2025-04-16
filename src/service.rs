use std::{
    hash::{DefaultHasher, Hasher},
    sync::Arc,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{Hash, Url, cache::CachableStorage, storage::Storages};

pub struct Service {
    storage: Arc<CachableStorage>,
}

#[allow(dead_code)]
impl Service {
    pub async fn new() -> Self {
        let storage = CachableStorage::new().await;
        Self {
            storage: Arc::new(storage),
        }
    }

    pub async fn process_post(&self, url: Url) -> Response {
        if let Some(code) = self.storage.get_key_by_value(&url).await {
            return axum::Json(code).into_response();
        };

        let is_url_a = Self::is_valid_public_url(&url.0);

        if !is_url_a {
            eprintln!("Invalid URL: {}", url.0);
            return (axum::http::StatusCode::NOT_FOUND, "Invalid URL").into_response();
        }

        let hash = Service::hash(&url);

        self.storage.insert(url, hash.clone()).await;

        axum::Json(hash).into_response()
    }

    pub async fn length(&self) -> Response {
        (StatusCode::OK, self.storage.length().await.to_string()).into_response()
    }

    pub async fn process_get(&self, code: u64) -> Response {
        match self.storage.get(code.into()).await {
            Some(url) => axum::response::Redirect::permanent(&url.0).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                "URL not found in our system",
            )
                .into_response(),
        }
    }

    pub fn is_valid_public_url(url: &str) -> bool {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return false;
        }

        let without_scheme = match url.split_once("://") {
            Some((_, rest)) => rest,
            None => return false,
        };

        let host = without_scheme
            .split('/')
            .next()
            .unwrap_or("")
            .split('@')
            .next_back()
            .unwrap_or("")
            .split(':')
            .next()
            .unwrap_or("");

        if host.parse::<std::net::IpAddr>().is_ok() {
            return false;
        }

        if !host.contains('.') {
            return false;
        }

        if !host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            return false;
        }

        true
    }

    pub fn hash(url: &Url) -> Hash {
        let mut hasher = DefaultHasher::new();
        std::hash::Hash::hash(&url, &mut hasher);
        Hash(hasher.finish())
    }
}
