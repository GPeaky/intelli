use crate::{
    config::Database,
    dtos::RegisterUserDto,
    error::{AppResult, UserError},
    repositories::{UserRepository, UserRepositoryTrait},
};
use argon2::{self, Config};
use axum::async_trait;
use chrono::Utc;
use dotenvy::var;
use rs_nanoid::standard;
use std::sync::Arc;

#[derive(Clone)]
pub struct UserService {
    pass_salt: Vec<u8>,
    db_conn: Arc<Database>,
    user_repo: UserRepository,
    argon2_config: argon2::Config<'static>,
}

#[async_trait]
pub trait UserServiceTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn register(&self, register: RegisterUserDto) -> AppResult<()>;
}

#[async_trait]
impl UserServiceTrait for UserService {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: db_conn.clone(),
            user_repo: UserRepository::new(db_conn),
            argon2_config: Config::default(),
            pass_salt: var("PASS_SALT").unwrap().as_bytes().to_owned(),
        }
    }

    async fn register(&self, register: RegisterUserDto) -> AppResult<()> {
        let user_exists = self.user_repo.user_exists(&register.email).await?;

        if user_exists {
            return Err(UserError::AlreadyExists)?;
        }

        self
            .db_conn
            .get_scylla()
            .query(
                "INSERT INTO intelli_api.users (id, username, password, email, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                (
                    standard::<16>().to_string(),
                    register.username,
                    argon2::hash_encoded(register.password.as_bytes(), &self.pass_salt, &self.argon2_config).unwrap(),
                    register.email,
                    Utc::now().timestamp(),
                    Utc::now().timestamp(),
                ),
            )
            .await
            .unwrap();

        Ok(())
    }
}