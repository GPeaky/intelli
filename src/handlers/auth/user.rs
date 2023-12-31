use crate::{
    dtos::{
        AuthResponse, EmailUser, FingerprintQuery, ForgotPasswordDto, LoginUserDto,
        PasswordChanged, RefreshResponse, RefreshTokenQuery, RegisterUserDto, ResetPassword,
        ResetPasswordDto, ResetPasswordQuery, TokenType, VerifyEmail,
    },
    entity::{Provider, UserExtension},
    error::{AppResult, CommonError, UserError},
    repositories::UserRepositoryTrait,
    services::{TokenServiceTrait, UserServiceTrait},
    states::AppState,
};
use chrono::{Duration, Utc};
use garde::Validate;
use ntex::web;

#[inline(always)]
pub(crate) async fn register(
    state: web::types::State<AppState>,
    form: web::types::Form<RegisterUserDto>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state.user_service.create(&form).await?;

    let token = state
        .token_service
        .generate_token(user_id, TokenType::Email)
        .await?;

    let template = VerifyEmail {
        verification_link: &format!(
            "https://intellitelemetry.live/auth/verify-email?token={}",
            token
        ),
    };

    let save_email_future = state.token_service.save_email_token(&token);

    let send_email_future =
        state
            .email_service
            .send_mail((&*form).into(), "Verify Email", template);

    tokio::try_join!(save_email_future, send_email_future)?;
    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub(crate) async fn login(
    state: web::types::State<AppState>,
    query: web::types::Query<FingerprintQuery>,
    form: web::types::Form<LoginUserDto>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state.user_repository.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if !user.active {
        return Err(UserError::NotVerified)?;
    }

    if user.provider != Provider::Local {
        return Err(UserError::GoogleLogin)?;
    }

    if !state
        .user_repository
        .validate_password(&form.password, &user.password.unwrap())?
    {
        return Err(UserError::InvalidCredentials)?;
    }

    let access_token_future = state
        .token_service
        .generate_token(user.id, TokenType::Bearer);

    let refresh_token_future = state
        .token_service
        .generate_refresh_token(&user.id, &query.fingerprint);

    let (access_token, refresh_token) =
        tokio::try_join!(access_token_future, refresh_token_future)?;

    let auth_response = &AuthResponse {
        access_token,
        refresh_token,
    };

    Ok(web::HttpResponse::Ok().json(auth_response))
}

#[inline(always)]
pub(crate) async fn refresh_token(
    state: web::types::State<AppState>,
    query: web::types::Query<RefreshTokenQuery>,
) -> AppResult<impl web::Responder> {
    let access_token = state
        .token_service
        .refresh_access_token(&query.refresh_token, &query.fingerprint)
        .await?;

    let refresh_response = &RefreshResponse { access_token };

    Ok(web::HttpResponse::Ok().json(refresh_response))
}

#[inline(always)]
pub(crate) async fn logout(
    req: web::HttpRequest,
    state: web::types::State<AppState>,
    query: web::types::Query<FingerprintQuery>,
) -> AppResult<impl web::Responder> {
    let user_id = req
        .extensions()
        .get::<UserExtension>()
        .ok_or(CommonError::InternalServerError)?
        .id;

    state
        .token_service
        .remove_refresh_token(&user_id, &query.fingerprint)
        .await?;

    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub(crate) async fn forgot_password(
    state: web::types::State<AppState>,
    form: web::types::Form<ForgotPasswordDto>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let Some(user) = state.user_repository.find_by_email(&form.email).await? else {
        return Err(UserError::NotFound)?;
    };

    if Utc::now().signed_duration_since(user.updated_at) > Duration::hours(1) {
        return Err(UserError::UpdateLimitExceeded)?;
    }

    let token = state
        .token_service
        .generate_token(user.id, TokenType::ResetPassword)
        .await?;

    let template = ResetPassword {
        reset_password_link: &format!(
            "https://intellitelemetry.live/auth/reset-password?token={}",
            token
        ),
    };

    let save_reset_password = state.token_service.save_reset_password_token(&token);

    let send_mail = state.email_service.send_mail(
        EmailUser {
            username: &user.username,
            email: &user.email,
        },
        "Reset Password",
        template,
    );

    tokio::try_join!(save_reset_password, send_mail)?;
    Ok(web::HttpResponse::Ok())
}

#[inline(always)]
pub async fn reset_password(
    query: web::types::Query<ResetPasswordQuery>,
    state: web::types::State<AppState>,
    form: web::types::Form<ResetPasswordDto>,
) -> AppResult<impl web::Responder> {
    if form.validate(&()).is_err() {
        return Err(CommonError::ValidationFailed)?;
    }

    let user_id = state
        .user_service
        .reset_password_with_token(&query.token, &form.password)
        .await?;

    let Some(user) = state.user_repository.find(&user_id).await? else {
        Err(UserError::NotFound)?
    };

    let template = PasswordChanged {};

    state
        .email_service
        .send_mail(
            EmailUser {
                username: &user.username,
                email: &user.email,
            },
            "Password Changed",
            template,
        )
        .await?;

    Ok(web::HttpResponse::Ok())
}
