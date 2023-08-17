use crate::{
    config::Database,
    dtos::{PreparedStatementsKey, UserStatements},
    entity::User,
    error::AppResult,
};
use axum::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserRepository {
    db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn find(&self, id: &i32) -> AppResult<User>;
    async fn find_by_email(&self, email: &str) -> AppResult<User>;
    async fn user_exists(&self, email: &str) -> AppResult<bool>;
    fn validate_password(&self, password: &str, hash: &str) -> bool;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
        }
    }

    // TODO: Check why not finding any user
    async fn find_by_email(&self, email: &str) -> AppResult<User> {
        let user = self
            .db_conn
            .scylla
            .execute(
                self.db_conn
                    .statements
                    .get(&PreparedStatementsKey::User(UserStatements::ByEmail))
                    .unwrap(),
                (email,),
            )
            .await?
            .single_row_typed::<User>()?;

        Ok(user)
    }

    async fn find(&self, id: &i32) -> AppResult<User> {
        let user = self
            .db_conn
            .scylla
            .execute(
                self.db_conn
                    .statements
                    .get(&PreparedStatementsKey::User(UserStatements::ById))
                    .unwrap(),
                (id,),
            )
            .await?
            .single_row_typed::<User>()?;

        Ok(user)
    }

    async fn user_exists(&self, email: &str) -> AppResult<bool> {
        let rows = self
            .db_conn
            .scylla
            .execute(
                self.db_conn
                    .statements
                    .get(&PreparedStatementsKey::User(UserStatements::EmailByEmail))
                    .unwrap(),
                (email,),
            )
            .await?
            .rows_num()?;

        Ok(rows > 0)
    }

    fn validate_password(&self, pwd: &str, hash: &str) -> bool {
        argon2::verify_encoded(hash, pwd.as_bytes()).unwrap()
    }
}
