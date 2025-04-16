#![allow(clippy::new_without_default)]
use std::sync::Arc;

use mini_moka::sync::Cache;

use crate::{Hash, Url, db::DB, storage::Storages};

pub struct CachableStorage {
    hash_to_url_cache: Cache<Hash, Arc<Url>>,
    url_to_hash_cache: Cache<Arc<Url>, Hash>,
    database: DB,
}

impl CachableStorage {
    pub async fn shutdown(&self) {
        self.database.shutdown().await
    }
}

impl Storages for CachableStorage {
    async fn new() -> Self {
        Self {
            hash_to_url_cache: Cache::new(10_000),
            url_to_hash_cache: Cache::new(10_000),
            database: DB::new().await,
        }
    }

    async fn get(&self, hash: Hash) -> Option<Url> {
        if let Some(url_arc) = self.hash_to_url_cache.get(&hash) {
            return Some((*url_arc).clone());
        }

        let db_result = self.database.get(hash.clone()).await;

        if let Some(url) = db_result {
            let url_arc = Arc::new(url.clone());
            self.hash_to_url_cache.insert(hash.clone(), url_arc.clone());
            self.url_to_hash_cache.insert(url_arc, hash);
            Some(url)
        } else {
            None
        }
    }

    async fn get_key_by_value(&self, url: &Url) -> Option<Hash> {
        let url_arc = Arc::new(url.clone());

        if let Some(hash) = self.url_to_hash_cache.get(&url_arc) {
            return Some(hash);
        }

        let db_result = self.database.get_key_by_value(url).await;

        if let Some(hash) = db_result {
            self.hash_to_url_cache.insert(hash.clone(), url_arc.clone());
            self.url_to_hash_cache.insert(url_arc, hash.clone());
            Some(hash)
        } else {
            None
        }
    }

    async fn insert(&self, url: Url, hash: Hash) {
        let url_arc = Arc::new(url.clone());
        self.hash_to_url_cache.insert(hash.clone(), url_arc.clone());
        self.url_to_hash_cache.insert(url_arc, hash.clone());

        self.database.insert(url, hash).await;
    }

    async fn length(&self) -> usize {
        self.database.length().await
    }
}
