use std::path::PathBuf;

use crate::{
    config::Config,
    db,
    logger::create_logger,
    model::auth::{
        InvalidRegisterRequest, LoginError, LoginRequest, LoginResponse, RegisterRequest,
    },
    model::image::CreateImageError,
    model::image::CreateImageRequest,
    model::image::CreateImageResponse,
    model::image::UploadImageError,
    server,
    services::{auth, AuthService, DefaultImageService, ImageService},
    util::random_string,
};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use actix_web::{test, web::ServiceConfig, App};
use async_trait::async_trait;
use auth::DefaultAuthService;
use db::image::NewImage;
use slog::Logger;
use uuid::Uuid;

const TEST_IMAGE: &[u8] = include_bytes!("../../test_data/test.png");

/// A proxy service for debugging.
#[derive(Clone)]
struct TestAuthService(Box<dyn AuthService>);

#[async_trait]
impl AuthService for TestAuthService {
    async fn register(
        &self,
        email: &str,
        password: &str,
    ) -> Result<(), crate::model::auth::RegisterError> {
        println!("(TestAuthService) register called: {} {}", email, password);
        // Checks or mocks here.
        self.0.register(email, password).await
    }

    async fn login(&self, email: &str, password: &str) -> Result<auth::Token, LoginError> {
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
#[derive(Clone)]
struct TestImageService(Box<dyn ImageService>);

#[async_trait(?Send)]
impl ImageService for TestImageService {
    async fn create_image(
        &self,
        app_user_id: Uuid,
        image: NewImage,
    ) -> Result<Uuid, CreateImageError> {
        println!(
            "(TestImageService) create image called: {} {}",
            app_user_id, image.title
        );
        // Checks or mocks here.
        self.0.create_image(app_user_id, image).await
    }

    async fn save_image(&self, id: Uuid, payload: Multipart) -> Result<(), UploadImageError> {
        println!("(TestImageService) save image called: {}", id);
        // Checks or mocks here.
        self.0.save_image(id, payload).await
    }

    async fn get_image(&self, id: Uuid) -> Result<NamedFile, std::io::Error> {
        println!("(TestImageService) get image called: {}", id);
        // Checks or mocks here.
        self.0.get_image(id).await
    }
}

pub fn configure_services(
    config: &Config,
    logger: Logger,
    pool: sqlx::PgPool,
) -> impl FnOnce(&mut ServiceConfig) {
    let c = config.clone();
    move |app: &mut ServiceConfig| {
        let auth_service = TestAuthService(Box::new(DefaultAuthService::new(
            &c,
            logger.clone(),
            pool.clone(),
        )));
        let image_service = TestImageService(Box::new(DefaultImageService::new(&c, logger, pool)));

        app.data::<Box<dyn AuthService>>(Box::new(auth_service));
        app.data::<Box<dyn ImageService>>(Box::new(image_service));
    }
}

/// Tests a whole procedure from register/login to picture upload/download.
#[actix_rt::test]
async fn picture_upload() {
    let mut config = Config::from_env().unwrap();
    config.image_storage_path = PathBuf::from("./.tmp_images");

    let mut app = test::init_service(
        App::new()
            .data(create_logger(&config))
            .configure(configure_services(
                &config,
                create_logger(&config),
                db::connect(&config).await.unwrap(),
            ))
            .configure(server::configure_routes(&config)),
    )
    .await;

    // Register and login
    let token: String;
    {
        let invalid_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&RegisterRequest {
                email: "invalid email 2131231".into(),
                password: "password".into(),
            })
            .to_request();

        let res = test::call_service(&mut app, invalid_req).await;
        assert!(
            res.status().as_u16() == 400,
            "got: {}",
            res.status().as_u16()
        );
        let invalid_req_data: InvalidRegisterRequest = test::read_body_json(res).await;
        assert!(invalid_req_data.error.contains("e-mail"));

        let email = format!("test_{}@asd.com", random_string(12));
        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&RegisterRequest {
                email: email.clone(),
                password: "password".into(),
            })
            .to_request();

        let register_res = test::call_service(&mut app, register_req).await;
        assert!(register_res.status() == 204);

        let wrong_password_req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&LoginRequest {
                email: email.clone(),
                password: "wrongPassword".into(),
            })
            .to_request();
        let wrong_password_res = test::call_service(&mut app, wrong_password_req).await;
        assert!(wrong_password_res.status() == 403);

        let wrong_email_req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&LoginRequest {
                email: "doesnt@exist.com".into(),
                password: "password".into(),
            })
            .to_request();
        let wrong_email_res = test::call_service(&mut app, wrong_email_req).await;
        assert!(wrong_email_res.status() == 404);

        let login_req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&LoginRequest {
                email: email.clone(),
                password: "password".into(),
            })
            .to_request();
        let login_res = test::call_service(&mut app, login_req).await;
        assert!(login_res.status() == 200);

        let login_res_data: LoginResponse = test::read_body_json(login_res).await;
        token = login_res_data.token;
    }

    // Image upload
    let image_id: Uuid;
    {
        let create_image_req = test::TestRequest::post()
            .uri("/images")
            .header("Authorization", format!("Bearer {}", token))
            .set_json(&CreateImageRequest {
                title: "test_image".to_string(),
                categories: Vec::new(),
                description: "asd".to_string().into(),
            })
            .to_request();
        let create_image_res = test::call_service(&mut app, create_image_req).await;
        assert!(
            create_image_res.status() == 200,
            "got {}, {:?}",
            create_image_res.status().as_u16(),
            test::read_body(create_image_res).await
        );

        let create_image_res_data: CreateImageResponse =
            test::read_body_json(create_image_res).await;
        image_id = create_image_res_data.id;

        // let mut image_upload_data = Vec::from(TEST_IMAGE);
        let mut image_upload_data = Vec::new();

        image_upload_data.extend("--test_image\r\n".bytes());
        image_upload_data.extend(
            r#"Content-Disposition: form-data; name="image"; filename="test_image.png""#.bytes(),
        );
        image_upload_data.extend("\r\n\r\n".bytes());
        image_upload_data.extend(TEST_IMAGE);
        image_upload_data.extend("\r\n--test_image--\r\n".bytes());

        let upload_image_req = test::TestRequest::post()
            .uri(&format!("/images/{}", image_id))
            .header("Authorization", format!("Bearer {}", token))
            .header("Content-Type", "multipart/form-data; boundary=test_image")
            .header("Content-Length", image_upload_data.len())
            .set_payload(image_upload_data)
            .to_request();
        let upload_image_res = test::call_service(&mut app, upload_image_req).await;

        assert!(
            upload_image_res.status() == 204,
            "got {}, {:?}",
            upload_image_res.status().as_u16(),
            test::read_body(upload_image_res).await
        );

        let download_image_req = test::TestRequest::get()
            .uri(&format!("/images/{}", image_id))
            .header("Authorization", format!("Bearer {}", token))
            .to_request();
        let download_image_res = test::call_service(&mut app, download_image_req).await;

        assert!(
            download_image_res.status() == 200,
            "got {}",
            download_image_res.status().as_u16(),
        );

        assert!(test::read_body(download_image_res).await == TEST_IMAGE);
    }
}
