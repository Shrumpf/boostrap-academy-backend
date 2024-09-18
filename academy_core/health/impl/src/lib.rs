use std::{sync::Arc, time::Duration};

use academy_cache_contracts::CacheService;
use academy_core_health_contracts::{HealthFeatureService, HealthStatus};
use academy_di::Build;
use academy_email_contracts::EmailService;
use academy_persistence_contracts::Database;
use academy_shared_contracts::time::TimeService;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tracing::error;

#[derive(Debug, Clone, Build)]
pub struct HealthFeatureServiceImpl<Time, Db, Cache, Email> {
    time: Time,
    db: Db,
    cache: Cache,
    email: Email,
    config: HealthFeatureConfig,
    #[state]
    state: Arc<State>,
}

#[derive(Debug, Clone)]
pub struct HealthFeatureConfig {
    pub cache_ttl: Duration,
}

#[derive(Debug, Default)]
struct State {
    cache: RwLock<Option<CachedStatus>>,
}

#[derive(Debug)]
struct CachedStatus {
    status: HealthStatus,
    timestamp: DateTime<Utc>,
}

impl<Time, Db, Cache, Email> HealthFeatureService
    for HealthFeatureServiceImpl<Time, Db, Cache, Email>
where
    Time: TimeService,
    Db: Database,
    Cache: CacheService,
    Email: EmailService,
{
    async fn get_status(&self) -> HealthStatus {
        let now = self.time.now();
        let cache_guard = self.state.cache.read().await;
        if let Some(cached) = cache_guard
            .as_ref()
            .filter(|c| now < c.timestamp + self.config.cache_ttl)
        {
            return cached.status;
        }
        drop(cache_guard);

        let mut cache_guard = self.state.cache.write().await;
        if let Some(cached) = cache_guard
            .as_ref()
            .filter(|c| now < c.timestamp + self.config.cache_ttl)
        {
            return cached.status;
        }

        let database = self
            .db
            .ping()
            .await
            .inspect_err(|err| error!("Failed to ping database: {err}"))
            .is_ok();

        let cache = self
            .cache
            .ping()
            .await
            .inspect_err(|err| error!("Failed to ping cache: {err}"))
            .is_ok();

        let email = self
            .email
            .ping()
            .await
            .inspect_err(|err| error!("Failed to ping smtp server: {err}"))
            .is_ok();

        let status = HealthStatus {
            database,
            cache,
            email,
        };

        cache_guard
            .insert(CachedStatus {
                status,
                timestamp: now,
            })
            .status
    }
}
