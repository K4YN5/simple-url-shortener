#![allow(clippy::new_without_default)]

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Executor, Pool, Sqlite};

use crate::storage::Storages;
use crate::{Hash, Url};

pub struct DB {
    pool: Pool<Sqlite>,
}

impl DB {
    pub async fn shutdown(&self) {
        self.pool.close().await;
        log::trace!("Shutting down database");
    }
}

impl Storages for DB {
    async fn new() -> Self
    where
        Self: Sized,
    {
        if !std::path::Path::new("./data").exists() {
            log::trace!("No Sqlite DB folder, creating a new one");
            std::fs::create_dir("./data").expect("Failed to create data directory");
        }

        if !std::path::Path::new("./data/db.sqlite").is_file() {
            log::trace!("No Sqlite DB file, creating a new one");
            std::fs::File::create("./data/db.sqlite").expect("Failed to create file");
        }

        log::trace!("Starting connection to Sqlite DB");

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(8))
            .idle_timeout(std::time::Duration::from_secs(60 * 5))
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    conn.execute("PRAGMA journal_mode = WAL;").await?;
                    conn.execute("PRAGMA foreign_keys = ON;").await?;
                    conn.execute("PRAGMA synchronous = NORMAL;").await?;
                    conn.execute("PRAGMA busy_timeout = 5000;").await?;
                    conn.execute("PRAGMA wal_autocheckpoint = 500;").await?;
                    Ok(())
                })
            })
            .connect("sqlite://./data/db.sqlite")
            .await
            .expect("Error connecting to the db, shouldn't happen");

        log::info!("Connected to Sqlite database succesfully");

        sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS mainstorage (
                id TEXT PRIMARY KEY NOT NULL, -- Store hash as TEXT for simplicity or INTEGER
                url TEXT NOT NULL UNIQUE      -- Ensure URLs are unique
            );
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to create table");

        log::trace!("Ran migrations on the Sqlite database");

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_url ON mainstorage(url);")
            .execute(&pool)
            .await
            .expect("Failed to create index 'idx_url'");

        log::trace!("Enabled Index on the db for faster lookups");

        Self { pool }
    }

    async fn get(&self, hash: Hash) -> Option<Url> {
        log::trace!("Searching URL from HASH {} in Sqlite", hash.0);
        let hash_str = hash.0.to_string();
        let find: Option<String> =
            sqlx::query_scalar!("SELECT url FROM mainstorage WHERE id = ?", hash_str)
                .fetch_optional(&self.pool)
                .await
                .expect("Error retrieving from the db");

        find.map(|v| v.into())
    }

    async fn get_key_by_value(&self, url: &Url) -> Option<Hash> {
        log::trace!("Searching HASH from URL {} in Sqlite", url.0);
        let find: Option<String> =
            sqlx::query_scalar!("SELECT id FROM mainstorage WHERE url = ?", url.0)
                .fetch_optional(&self.pool)
                .await
                .expect("Error retrieving from the db");

        find.map(|v| v.parse::<u64>().unwrap().into())
    }

    async fn insert(&self, url: Url, hash: Hash) {
        let url = url.0;
        let hash_str = hash.0.to_string();
        sqlx::query!(
            "INSERT OR IGNORE INTO mainstorage (id, url) VALUES (?, ?)",
            hash_str, // Bind the hash string
            url       // Bind the URL string
        )
        .execute(&self.pool)
        .await
        .expect("Error inserting to db");
    }

    async fn length(&self) -> usize {
        sqlx::query_scalar!(
            // Alias "total" is optional for scalar but good practice; macro infers from first col anyway
            "SELECT COUNT(*) as total FROM mainstorage" // Type annotation `_: i64` can be added for explicitness if needed:
                                                        // "SELECT COUNT(*) as total FROM mainstorage", // _: i64
        )
        .fetch_one(&self.pool) // Returns Result<i64, Error>
        .await
        .expect("Failed retrieving length")
        .try_into()
        .unwrap()
    }
}
