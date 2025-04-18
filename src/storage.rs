use crate::{SeqId, Url};

pub trait Storages: Send + Sync {
    async fn new() -> Self
    where
        Self: Sized;

    /// Get the Url for a id
    async fn get(&self, id: SeqId) -> Option<Url>;
    /// Get the id for a Url
    async fn get_key_by_value(&self, url: &Url) -> Option<SeqId>;

    /// Insert a Url into the storage, returns it's id
    async fn insert(&self, url: Url) -> SeqId;

    /// Returns the current length of the storage: number of different Urls
    async fn length(&self) -> usize;
}
