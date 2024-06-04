use std::time::Duration;

use academy_cache_contracts::CacheService;
use bb8_redis::{
    bb8::Pool,
    redis::{self, AsyncCommands},
    RedisConnectionManager,
};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub struct ValkeyCache {
    pool: Pool<RedisConnectionManager>,
}

#[derive(Debug)]
pub struct ValkeyCacheConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

impl ValkeyCache {
    pub async fn connect(config: &ValkeyCacheConfig) -> anyhow::Result<Self> {
        let manager = RedisConnectionManager::new(config.url.as_str())?;
        let pool = Pool::builder()
            .max_size(config.max_connections)
            .min_idle(config.min_connections)
            .connection_timeout(config.acquire_timeout)
            .idle_timeout(config.idle_timeout)
            .max_lifetime(config.max_lifetime)
            .build(manager)
            .await?;

        Ok(Self { pool })
    }

    #[cfg(feature = "dummy")]
    pub async fn dummy() -> Self {
        let manager = RedisConnectionManager::new("redis://dummy").unwrap();
        Self {
            pool: Pool::builder().build_unchecked(manager),
        }
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        redis::cmd("FLUSHDB")
            .exec_async(&mut *conn)
            .await
            .map_err(Into::into)
    }
}

impl CacheService for ValkeyCache {
    async fn get<T: DeserializeOwned + 'static>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let mut conn = self.pool.get().await?;
        conn.get::<_, Option<Vec<u8>>>(key)
            .await
            .map_err(Into::into)
            .and_then(|result| {
                result
                    .map(|data| postcard::from_bytes(&data))
                    .transpose()
                    .map_err(Into::into)
            })
    }

    async fn set<T: Serialize + Sync + 'static>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> anyhow::Result<()> {
        let value = postcard::to_stdvec(&value)?;
        let mut conn = self.pool.get().await?;
        if let Some(ttl) = ttl {
            conn.pset_ex(key, value, ttl.as_millis().try_into()?).await
        } else {
            conn.set(key, value).await
        }
        .map_err(Into::into)
    }

    async fn remove(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        conn.del(key).await.map_err(Into::into)
    }

    async fn ping(&self) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        redis::cmd("PING")
            .exec_async(&mut *conn)
            .await
            .map_err(Into::into)
    }
}
