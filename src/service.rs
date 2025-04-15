use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{Code, Storage, Url};

pub struct Service {
    storage: Arc<Storage>,
}

#[allow(dead_code)]
impl Service {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: Arc::new(storage),
        }
    }

    pub fn process_post(&self, url: Url) -> Response {
        if let Some(code) = self.storage.inverted_get(&url.0) {
            return axum::Json(code).into_response();
        };

        let is_url_a = Self::is_valid_public_url(&url.0);

        if !is_url_a {
            eprintln!("Invalid URL: {}", url.0);
            return (axum::http::StatusCode::NOT_FOUND, "Invalid URL").into_response();
        }

        let code: Code = self.storage.insert(&url.0);

        axum::Json(code).into_response()
    }

    pub fn length(&self) -> Response {
        (StatusCode::OK, self.storage.length().to_string()).into_response()
    }

    pub fn process_get(&self, code: u64) -> Response {
        match self.storage.get(code) {
            Some(url) => axum::response::Redirect::permanent(&url.0).into_response(),
            None => (
                axum::http::StatusCode::NOT_FOUND,
                "URL not found in our system",
            )
                .into_response(),
        }
    }

    fn is_strict_valid_url(s: &str) -> bool {
        if let Ok(url) = url::Url::parse(s) {
            matches!(url.scheme(), "http" | "https")
                && url.has_host()
                && url.host_str().map(|h| h.contains('.')).unwrap_or(false)
        } else {
            false
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

    pub fn hash(url: Url) -> Code {
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        Code(hasher.finish())
    }
}
