use crate::{
    config::Database,
    repositories::{ChampionshipRepository, F123Repository, UserRepository, UserRepositoryTrait},
    services::{
        ChampionshipService, F123Service, TokenService, TokenServiceTrait, UserService,
        UserServiceTrait,
    },
};
use std::sync::Arc;

pub type UserState = Arc<UserStateInner>;

pub struct UserStateInner {
    pub user_service: UserService,
    pub user_repository: UserRepository,
    pub token_service: TokenService,
    pub championship_service: ChampionshipService,
    pub championship_repository: ChampionshipRepository,
    pub f123_service: F123Service,
    pub f123_repository: F123Repository,
}

impl UserStateInner {
    pub async fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            user_service: UserService::new(db_conn),
            f123_service: F123Service::new(db_conn),
            f123_repository: F123Repository::new(db_conn),
            user_repository: UserRepository::new(db_conn),
            token_service: TokenService::new(db_conn),
            championship_service: ChampionshipService::new(db_conn).await,
            championship_repository: ChampionshipRepository::new(db_conn),
        }
    }
}
