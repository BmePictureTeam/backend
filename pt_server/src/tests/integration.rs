use crate::{config::Config, db, logger::create_logger, server::{self, routes::user::{InvalidRegisterRequest, LoginRequest, RegisterRequest}}, services::{app_user, auth, AppUserService, AuthService}, util::random_string};
use actix_web::{test, web::ServiceConfig, App};
use app_user::DefaultAppUserService;
use async_trait::async_trait;
use auth::DefaultAuthService;
use slog::Logger;

/// A proxy service for debugging.
struct TestAuthService(Box<dyn AuthService>);

#[async_trait]
impl AuthService for TestAuthService {
    async fn login(&self, email: &str, password: &str) -> Result<auth::Token, auth::LoginError> {
        println!("(TestAuthService) login called: {} {}", email, password);
        // Checks or mocks here.
        self.0.login(email, password).await
    }

    async fn validate_token(&self, token: &str) -> Result<auth::UserInfo, jwt::errors::Error> {
        println!("(TestAuthService) validate token called: {}", token);
        // Checks or mocks here.
        self.0.validate_token(token).await
    }
}

/// A proxy service for debugging.
struct TestAppUserService(Box<dyn AppUserService>);

#[async_trait]
impl AppUserService for TestAppUserService {
    async fn register(&self, email: &str, password: &str) -> Result<(), app_user::RegisterError> {
        println!(
            "(TestAppUserService) register called: {} {}",
            email, password
        );
        self.0.register(email, password).await
    }
}

pub fn configure_services(
    config: &Config,
    logger: Logger,
    pool: sqlx::PgPool,
) -> impl FnOnce(&mut ServiceConfig) {
    let c = config.clone();
    move |app: &mut ServiceConfig| {
        let app_user_service = TestAppUserService(Box::new(DefaultAppUserService::new(
            &c,
            logger.clone(),
            pool.clone(),
        )));
        let auth_service = TestAuthService(Box::new(DefaultAuthService::new(&c, logger, pool)));

        app.data::<Box<dyn AppUserService>>(Box::new(app_user_service));
        app.data::<Box<dyn AuthService>>(Box::new(auth_service));
    }
}

/// Registration and login
#[actix_rt::test]
async fn test_user() {
    let config = Config::from_env().unwrap();

    let mut app = test::init_service(
        App::new()
            .configure(configure_services(
                &config,
                create_logger(&config),
                db::connect(&config).await.unwrap(),
            ))
            .configure(server::configure_routes(&config)),
    )
    .await;

    {
        let req = test::TestRequest::post()
            .uri("/user/register")
            .set_json(&RegisterRequest {
                email: "invalid email 2131231".into(),
                password: "password".into(),
            })
            .to_request();

        let res = test::call_service(&mut app, req).await;
        assert!(res.status().as_u16() == 400);
        let invalid_req: InvalidRegisterRequest = test::read_body_json(res).await;
        assert!(invalid_req.error.contains("e-mail"));
    }
    {
        let email = format!("test_{}@asd.com", random_string(12));
        let register_req = test::TestRequest::post()
            .uri("/user/register")
            .set_json(&RegisterRequest {
                email: email.clone(),
                password: "password".into(),
            })
            .to_request();

        let register_res = test::call_service(&mut app, register_req).await;
        assert!(register_res.status() == 204);

        let login_req = test::TestRequest::post()
        .uri("/user/login")
        .set_json(&LoginRequest {
            email: email.clone(),
            password: "password".into(),
        })
        .to_request();
        let login_res = test::call_service(&mut app, login_req).await;
        assert!(login_res.status() == 200);

        let wrong_password_req = test::TestRequest::post()
        .uri("/user/login")
        .set_json(&LoginRequest {
            email: email.clone(),
            password: "wrongPassword".into(),
        })
        .to_request();
        let wrong_password_res = test::call_service(&mut app, wrong_password_req).await;
        assert!(wrong_password_res.status() == 403);

        let wrong_email_req = test::TestRequest::post()
        .uri("/user/login")
        .set_json(&LoginRequest {
            email: "doesnt@exist.com".into(),
            password: "password".into(),
        })
        .to_request(); 
        let wrong_email_res = test::call_service(&mut app, wrong_email_req).await;
        assert!(wrong_email_res.status() == 404);
    }
}
