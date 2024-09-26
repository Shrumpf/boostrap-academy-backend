use academy_cache_valkey::{ValkeyCache, ValkeyCacheConfig};
use academy_config::CacheConfig;

/// Connect to Valkey
pub async fn connect(config: &CacheConfig) -> anyhow::Result<ValkeyCache> {
    ValkeyCache::connect(&ValkeyCacheConfig {
        url: config.url.clone(),
        max_connections: config.max_connections,
        min_connections: config.min_connections,
        acquire_timeout: config.acquire_timeout.into(),
        idle_timeout: config.idle_timeout.map(Into::into),
        max_lifetime: config.max_lifetime.map(Into::into),
    })
    .await
}
