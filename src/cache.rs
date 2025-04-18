#![allow(clippy::new_without_default)]
use std::sync::Arc;

use mini_moka::sync::Cache;

use crate::{SeqId, Url, db::DB, storage::Storages};

pub struct CachableStorage {
    id_to_url_cache: Cache<SeqId, Arc<Url>>,
    url_to_id_cache: Cache<Arc<Url>, SeqId>,
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
            id_to_url_cache: Cache::new(10_000),
            url_to_id_cache: Cache::new(10_000),
            database: DB::new().await,
        }
    }

    async fn get(&self, id: SeqId) -> Option<Url> {
        if let Some(url_arc) = self.id_to_url_cache.get(&id) {
            return Some((*url_arc).clone());
        }

        let db_result = self.database.get(id.clone()).await;

        if let Some(url) = db_result {
            let url_arc = Arc::new(url.clone());
            self.id_to_url_cache.insert(id.clone(), url_arc.clone());
            self.url_to_id_cache.insert(url_arc, id);
            Some(url)
        } else {
            None
        }
    }

    async fn get_key_by_value(&self, url: &Url) -> Option<SeqId> {
        let url_arc = Arc::new(url.clone());

        if let Some(id) = self.url_to_id_cache.get(&url_arc) {
            return Some(id);
        }

        let db_result = self.database.get_key_by_value(url).await;

        if let Some(id) = db_result {
            self.id_to_url_cache.insert(id.clone(), url_arc.clone());
            self.url_to_id_cache.insert(url_arc, id.clone());
            Some(id)
        } else {
            None
        }
    }

    async fn insert(&self, url: Url) -> SeqId {
        let id = self.database.insert(url.clone()).await;

        let url_arc = Arc::new(url);

        self.id_to_url_cache.insert(id.clone(), url_arc.clone());
        self.url_to_id_cache.insert(url_arc, id.clone());

        id
    }

    async fn length(&self) -> usize {
        self.database.length().await
    }
}
