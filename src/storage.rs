use crate::{SeqId, Url};

pub trait Storages: Send + Sync {
    async fn new() -> Self
    where
        Self: Sized;

    async fn get(&self, id: SeqId) -> Option<Url>;
    async fn get_key_by_value(&self, url: &Url) -> Option<SeqId>;

    async fn insert(&self, url: Url) -> SeqId;

    async fn length(&self) -> usize;
}
