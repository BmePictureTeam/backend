use crate::{
    config::Config,
    model::auth::{
        InvalidRegisterRequest, LoginError, LoginRequest, LoginResponse, RegisterError,
        RegisterRequest,
    },
    model::error::GenericError,
    services::AuthService,
};
use actix_web::{
    post,
    web::{self, ServiceConfig},
    HttpResponse,
};
use aide::openapi::v3::macros::api;
use aide::openapi::v3::macros::api::define;

const TAG_NAME: &str = "auth";

define::tag! {
    name(TAG_NAME),
    description("Authentication and authorization routes"),
    display_name("Auth")
}

/// Login for existing users.
#[api]
#[post("/auth/login")]
#[tag(TAG_NAME)]
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
            LoginError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
        },
    }
}

/// Register endpoint for new users.
#[api]
#[post("/auth/register")]
#[tag(TAG_NAME)]
#[response(204)]
#[response(400, InvalidRegisterRequest)]
async fn register(
    req: web::Json<RegisterRequest>,
    auth_service: web::Data<Box<dyn AuthService>>,
) -> HttpResponse {
    match auth_service.register(&req.email, &req.password).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => match err {
            RegisterError::EmailExists
            | RegisterError::InvalidEmail
            | RegisterError::InvalidPassword(_) => {
                HttpResponse::BadRequest().json(InvalidRegisterRequest {
                    error: err.to_string(),
                })
            }
            RegisterError::Unexpected => {
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
