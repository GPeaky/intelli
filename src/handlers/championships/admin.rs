use crate::dtos::{ChampionshipIdPath, UserIdPath};
use crate::error::CommonError;
use crate::{
    error::{AppResult, ChampionshipError},
    states::AppState,
};
use garde::Validate;
use ntex::web;

#[inline(always)]
pub async fn user_championships(
    state: web::types::State<AppState>,
    path: web::types::Path<UserIdPath>,
) -> AppResult<impl web::Responder> {
    if path.validate(&()).is_err() {
        Err(CommonError::ValidationFailed)?
    }

    let championships = state.championship_repository.find_all(&path.id).await?;
    Ok(web::HttpResponse::Ok().json(&championships))
}

#[inline(always)]
pub async fn delete_championship(
    state: web::types::State<AppState>,
    path: web::types::Path<ChampionshipIdPath>,
) -> AppResult<impl web::Responder> {
    let Some(championship) = state.championship_repository.find(&path.id).await? else {
        Err(ChampionshipError::NotFound)?
    };

    state.championship_service.delete(&championship.id).await?;
    Ok(web::HttpResponse::Ok())
}
