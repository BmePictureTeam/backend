use crate::{
    config::Config,
    model::error::GenericError,
    services::{app_user::RegisterError, auth::LoginError, AppUserService, AuthService},
};
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse,
};
use aide::openapi::v3::macros::api;

#[api]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[api]
#[serde(rename_all = "camelCase")]
pub(crate) struct LoginResponse {
    pub(crate) token: String,
}

/// Login for existing users.
#[api]
#[post("/user/login")]
#[response(200, LoginResponse)]
#[response(status(404), desc("the user was not found"))]
#[response(status(403), desc("incorrect password"))]
async fn login(
    req: web::Json<LoginRequest>,
    auth_service: web::Data<Box<dyn AuthService>>,
) -> HttpResponse {
    match auth_service.login(&req.email, &req.password).await {
        Ok(token) => HttpResponse::Ok().json(LoginResponse { token }),
        Err(err) => match err {
            LoginError::UserNotFound => HttpResponse::NotFound().finish(),
            LoginError::IncorrectPassword => HttpResponse::Forbidden().finish(),
            LoginError::UnexpectedError => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
        },
    }
}

#[api]
#[serde(rename_all = "camelCase")]
pub(crate) struct RegisterRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[api]
#[serde(rename_all = "camelCase")]
pub(crate) struct InvalidRegisterRequest {
    pub(crate) error: String,
}

#[api]
#[post("/user/register")]
#[response(204)]
#[response(400, InvalidRegisterRequest)]
async fn register(
    req: web::Json<RegisterRequest>,
    user_service: web::Data<Box<dyn AppUserService>>,
) -> HttpResponse {
    match user_service.register(&req.email, &req.password).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => match err {
            RegisterError::EmailExists
            | RegisterError::InvalidEmail
            | RegisterError::InvalidPassword(_) => {
                HttpResponse::BadRequest().json(InvalidRegisterRequest {
                    error: err.to_string(),
                })
            }
            RegisterError::UnexpectedError => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
        },
    }
}

pub fn configure_routes(_config: &Config) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut ServiceConfig| {
        app.service(register);
        app.service(login);
    }
}
