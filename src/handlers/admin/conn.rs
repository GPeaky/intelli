use crate::{error::AppResult, repositories::UserRepositoryTrait, states::UserState};
use axum::{extract::State, Json};

#[inline(always)]
pub async fn pool_status(State(state): State<UserState>) -> AppResult<Json<(u32, u32)>> {
    let (redis, pg) = state.user_repository.active_pools();

    Ok(Json((redis, pg)))
}