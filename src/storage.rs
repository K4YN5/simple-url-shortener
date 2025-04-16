use crate::{Hash, Url};

pub trait Storages: Send + Sync {
    async fn new() -> Self
    where
        Self: Sized;

    /// Get the Url for a hash
    async fn get(&self, hash: Hash) -> Option<Url>;
    /// Get the hash for a Url
    async fn get_key_by_value(&self, url: &Url) -> Option<Hash>;

    /// Insert a Url into the storage, returns it's hash
    async fn insert(&self, url: Url, hash: Hash);

    /// Returns the current length of the storage: number of different Urls
    async fn length(&self) -> usize;
}
