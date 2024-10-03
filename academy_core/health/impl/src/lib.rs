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
    #[di(default)]
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

#[derive(Debug, Clone, Copy)]
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

        let status_if_not_expired = |cached: Option<CachedStatus>| {
            cached
                .filter(|c| now < c.timestamp + self.config.cache_ttl)
                .map(|c| c.status)
        };

        if let Some(status) = status_if_not_expired(*self.state.cache.read().await) {
            return status;
        }

        let mut cache_guard = self.state.cache.write().await;
        if let Some(status) = status_if_not_expired(*cache_guard) {
            return status;
        }

        let database = async {
            self.db
                .ping()
                .await
                .inspect_err(|err| error!("Failed to ping database: {err}"))
                .is_ok()
        };

        let cache = async {
            self.cache
                .ping()
                .await
                .inspect_err(|err| error!("Failed to ping cache: {err}"))
                .is_ok()
        };

        let email = async {
            self.email
                .ping()
                .await
                .inspect_err(|err| error!("Failed to ping smtp server: {err}"))
                .is_ok()
        };

        let (database, cache, email) = tokio::join!(database, cache, email);

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
