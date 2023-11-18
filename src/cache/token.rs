use crate::{
    config::{constants::*, Database},
    dtos::TokenType,
    error::AppResult,
};
use bb8_redis::redis::AsyncCommands;
use core::panic;
use std::sync::Arc;

pub struct TokenCache {
    db: Arc<Database>,
}

impl TokenCache {
    pub fn new(db: &Arc<Database>) -> Self {
        Self { db: db.clone() }
    }

    #[inline(always)]
    pub async fn set_token(&self, token: &str, token_type: &TokenType) -> AppResult<()> {
        if token_type == &TokenType::RefreshBearer {
            panic!("Refresh token must have a fingerprint");
        }

        let mut conn = self.db.redis.get().await?;

        conn.set_ex(
            &format!("{}:{token}", token_type.base_key()),
            1,
            GENERIC_TOKEN_EXPIRATION,
        )
        .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn get_token(&self, token: &str, token_type: &TokenType) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.get(&format!("{}:{}", token_type.base_key(), token))
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn set_refresh_token(&self, token: &str, fingerprint: &str) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.set_ex(
            &format!("{}:{}", TokenType::RefreshBearer.base_key(), fingerprint),
            token,
            REFRESH_TOKEN_EXPIRATION,
        )
        .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn get_refresh_token(&self, user_id: &i32, fingerprint: &str) -> AppResult<String> {
        let mut conn = self.db.redis.get().await?;

        let token = conn
            .get(&format!(
                "{}:{}:{}",
                TokenType::RefreshBearer.base_key(),
                user_id,
                fingerprint
            ))
            .await?;

        Ok(token)
    }

    #[inline(always)]
    pub async fn remove_token(&self, token: &str, token_type: &TokenType) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.del(&format!("{}:{}", token_type.base_key(), token))
            .await?;

        Ok(())
    }

    #[inline(always)]
    pub async fn remove_refresh_token(&self, user_id: &i32, fingerprint: &str) -> AppResult<()> {
        let mut conn = self.db.redis.get().await?;

        conn.del(&format!(
            "{}:{}:{}",
            TokenType::RefreshBearer.base_key(),
            user_id,
            fingerprint
        ))
        .await?;

        Ok(())
    }
}
