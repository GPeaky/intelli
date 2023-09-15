use crate::{
    entity::User,
    error::{AppResult, UserError},
    repositories::UserRepositoryTrait,
    services::UserServiceTrait,
    states::SafeUserState,
};
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Extension,
};
use hyper::StatusCode;

// TODO: Add admin user handlers
#[inline(always)]
pub async fn delete_user(
    State(state): State<SafeUserState>,
    Path(id): Path<u32>,
    Extension(user): Extension<User>,
) -> AppResult<Response> {
    let path_user = state.user_repository.find(&id).await?;

    if path_user.id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.delete_user(&id).await?;

    Ok(StatusCode::OK.into_response())
}

// TODO: Disable a user by id
#[inline(always)]
pub async fn disable_user(
    State(state): State<SafeUserState>,
    Path(id): Path<u32>,
    Extension(user): Extension<User>,
) -> AppResult<Response> {
    let path_user = state.user_repository.find(&id).await?;

    if path_user.active.eq(&false) {
        Err(UserError::AlreadyInactive)?
    }

    if path_user.id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.deactivate_user(&id).await?;

    Ok(StatusCode::OK.into_response())
}

// TODO: Enable a user by id
#[inline(always)]
pub async fn enable_user(
    State(state): State<SafeUserState>,
    Path(id): Path<u32>,
    Extension(user): Extension<User>,
) -> AppResult<Response> {
    let path_user = state.user_repository.find(&id).await?;

    if path_user.active.eq(&true) {
        Err(UserError::AlreadyActive)?
    }

    if path_user.id.eq(&user.id) {
        Err(UserError::AutoDelete)?
    }

    state.user_service.activate_user(&path_user.id).await?;
    Ok(StatusCode::OK.into_response())
}
