use super::EntityCache;
use crate::{
    config::Database,
    entity::User,
    error::{AppResult, CacheError},
};
use axum::async_trait;
use bb8_redis::redis::AsyncCommands;
use rkyv::{Deserialize, Infallible};
use std::sync::Arc;
use tracing::error;

const USER_PREFIX: &str = "user";

pub struct UserCache {
    db: Arc<Database>,
}

impl UserCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }
}

#[async_trait]
impl EntityCache for UserCache {
    type Entity = User;

    #[inline(always)]
    async fn get(&self, id: &i32) -> AppResult<Option<Self::Entity>> {
        let mut conn = self.db.redis.get().await?;
        let user = conn
            .get::<_, Vec<u8>>(&format!("{USER_PREFIX}:{}", id))
            .await?;

        if user.is_empty() {
            return Ok(None);
        }

        let archived = unsafe { rkyv::archived_root::<Self::Entity>(&user) };

        // TODO: Check a better way ton handle this
        let Ok(user) = archived.deserialize(&mut Infallible) else {
            error!("Failed to deserialize user from cache");
            Err(CacheError::Deserialize)?
        };

        Ok(Some(user))
    }

    #[inline(always)]
    async fn set(&self, entity: &Self::Entity) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        // TODO: Check a better way ton handle this
        let Ok(bytes) = rkyv::to_bytes::<_, 128>(entity) else {
            error!("Failed to serialize user to cache");
            Err(CacheError::Serialize)?
        };

        conn.set_ex::<&str, &[u8], ()>(
            &format!("{USER_PREFIX}: {}", entity.id),
            &bytes[..],
            Self::EXPIRATION,
        )
        .await?;

        Ok(())
    }
}
