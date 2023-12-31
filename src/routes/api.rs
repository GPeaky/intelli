use crate::{
    handlers::{
        auth::{
            callback, forgot_password, login, logout, refresh_token, register, reset_password,
            verify_email,
        },
        championships::{
            add_user, all_championships, create_championship, get_championship, remove_user,
            session_socket, socket_status, start_socket, stop_socket, update,
        },
        heartbeat,
        intelli_app::latest_release,
        user::{update_user, user_data},
    },
    middlewares::Authentication,
};
use ntex::web;

#[inline(always)]
pub(crate) fn api_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/google/callback", web::get().to(callback))
            .route("/register", web::post().to(register))
            .route("/login", web::post().to(login))
            .route("/refresh", web::get().to(refresh_token))
            .route("/verify/email", web::get().to(verify_email))
            .route("/forgot-password", web::post().to(forgot_password))
            .route("/reset-password", web::post().to(reset_password)),
    );

    cfg.service(
        web::resource("/logout")
            .route(web::get().to(logout))
            .wrap(Authentication),
    );

    cfg.service(
        web::scope("/user")
            .route("", web::put().to(update_user))
            .route("/data", web::get().to(user_data))
            .wrap(Authentication),
    );

    cfg.service(
        web::scope("/intelli-app").route("/releases/latest", web::get().to(latest_release)),
    );

    cfg.service(
        web::scope("/championships")
            .route("", web::post().to(create_championship))
            .route("/all", web::get().to(all_championships))
            .route("/{id}", web::get().to(get_championship))
            .route("/{id}", web::put().to(update))
            .route("/{id}/user/add", web::put().to(add_user))
            .route("/{id}/user/{user_id}", web::delete().to(remove_user))
            .route("/{id}/socket/start", web::get().to(start_socket))
            .route("/{id}/socket/status", web::get().to(socket_status))
            .route("/{id}/socket/stop", web::get().to(stop_socket))
            .wrap(Authentication),
    );

    cfg.route("/heartbeat", web::get().to(heartbeat));

    cfg.route(
        "/web_socket/championship/{id}",
        web::get().to(session_socket),
    );
}
