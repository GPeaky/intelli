use crate::{cache::RedisCache, config::Database, error::AppResult};

#[derive(Clone)]
pub struct SavedSessionService {
    #[allow(unused)]
    cache: RedisCache,
    #[allow(unused)]
    db_conn: Database,
    // saved_session_repo: SavedSessionRepository,
}

impl SavedSessionService {
    pub fn new(db_conn: &Database, cache: &RedisCache) -> Self {
        Self {
            cache: cache.clone(),
            db_conn: db_conn.clone(),
            // saved_session_repo: SavedSessionRepository::new(db_conn, cache),
        }
    }

    #[allow(unused)]
    pub async fn create(&self) -> AppResult<()> {
        let id = fastrand::i32(800000000..900000000);

        let conn = self.db_conn.pg.get().await?;
        let save_session_stmt = conn
            .prepare_cached(
                r#"
                    INSERT INTO saved_session (id)
                    VALUES ($1)
                "#,
            )
            .await?;

        conn.execute(&save_session_stmt, &[&id]).await?;

        Ok(())
    }
}
