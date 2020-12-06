use std::path::PathBuf;

use crate::{
    config::Config,
    db,
    logger::create_logger,
    model::auth::{
        InvalidRegisterRequest, LoginError, LoginRequest, LoginResponse, RegisterRequest,
    },
    model::image::*,
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
use sqlx::query;
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
        // Checks or mocks here.
        self.0.register(email, password).await
    }

    async fn login(&self, email: &str, password: &str) -> Result<auth::Token, LoginError> {
        // Checks or mocks here.
        self.0.login(email, password).await
    }

    async fn validate_token(&self, token: &str) -> Result<auth::UserInfo, jwt::errors::Error> {
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
        categories: &[Uuid],
    ) -> Result<Uuid, CreateImageError> {
        // Checks or mocks here.
        self.0.create_image(app_user_id, image, categories).await
    }

    async fn save_image(&self, id: Uuid, payload: Multipart) -> Result<(), UploadImageError> {
        // Checks or mocks here.
        self.0.save_image(id, payload).await
    }

    async fn get_image(&self, id: Uuid) -> Result<NamedFile, std::io::Error> {
        // Checks or mocks here.
        self.0.get_image(id).await
    }

    async fn search_images(
        &self,
        search: Option<&str>,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<
        Vec<(db::image::Image, Vec<db::category::Category>)>,
        crate::model::image::SearchImagesError,
    > {
        // Checks or mocks here.
        self.0.search_images(search, offset, limit).await
    }

    async fn rate_image(
        &self,
        image_id: Uuid,
        app_user_id: Uuid,
        rating: u32,
    ) -> Result<(), crate::model::image::RateImageError> {
        // Checks or mocks here.
        self.0.rate_image(image_id, app_user_id, rating).await
    }

    async fn get_image_ratings(
        &self,
        image_id: Uuid,
    ) -> Result<Vec<db::rating::Rating>, crate::model::image::GetImageRatingsError> {
        // Checks or mocks here.
        self.0.get_image_ratings(image_id).await
    }

    async fn get_categories(
        &self,
    ) -> Result<Vec<db::category::CategoryExt>, crate::model::image::GetCategoriesError> {
        // Checks or mocks here.
        self.0.get_categories().await
    }

    async fn create_category(
        &self,
        name: &str,
    ) -> Result<Uuid, crate::model::image::CreateCategoryError> {
        // Checks or mocks here.
        self.0.create_category(name).await
    }

    async fn rename_category(
        &self,
        id: Uuid,
        name: &str,
    ) -> Result<(), crate::model::image::RenameCategoryError> {
        // Checks or mocks here.
        self.0.rename_category(id, name).await
    }

    async fn delete_category(&self, id: Uuid) -> Result<(), DeleteCategoryError> {
        // Checks or mocks here.
        self.0.delete_category(id).await
    }

    async fn get_image_info(&self, id: Uuid) -> Result<(db::image::Image, Vec<db::category::Category>), GetImageInfoError> {
        // Checks or mocks here.
        self.0.get_image_info(id).await
    }

    async fn get_user_ratings(&self) -> Result<Vec<UserRating>, GetUserRatingsError> {
        // Checks or mocks here.
        self.0.get_user_ratings().await
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
async fn whole_app() {
    let mut config = Config::from_env().unwrap();
    config.image_storage_path = PathBuf::from("./.tmp_images");

    let pool = db::connect(&config).await.unwrap();

    // Setup admin user.
    let _ = DefaultAuthService::new(&config, create_logger(&config), pool.clone())
        .register("admin@admin.admin", "admin")
        .await;

    query!(
        r#"
        UPDATE app_user
        SET is_admin = TRUE
        WHERE 
            app_user.email = 'admin@admin.admin'
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    let mut app = test::init_service(
        App::new()
            .data(create_logger(&config))
            .configure(configure_services(
                &config,
                create_logger(&config),
                pool.clone(),
            ))
            .configure(server::configure_routes(&config)),
    )
    .await;

    // Create categories.
    let category_id;
    {
        let login_req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&LoginRequest {
                email: "admin@admin.admin".into(),
                password: "admin".into(),
            })
            .to_request();
        let login_res = test::call_service(&mut app, login_req).await;
        assert!(login_res.status() == 200);

        let login_res_data: LoginResponse = test::read_body_json(login_res).await;
        let token = login_res_data.token;

        let get_categories_req = test::TestRequest::get()
            .uri("/categories")
            .header("Authorization", format!("Bearer {}", token))
            .to_request();

        let res: GetCategoriesResponse =
            test::read_response_json(&mut app, get_categories_req).await;
        let category_count = res.categories.len();

        let create_category_req = test::TestRequest::post()
            .uri("/categories")
            .header("Authorization", format!("Bearer {}", token))
            .set_json(&CreateCategoryRequest {
                name: "test_category".into(),
            })
            .to_request();

        let new_category_res: CreateCategoryResponse =
            test::read_response_json(&mut app, create_category_req).await;
        category_id = new_category_res.id;

        let get_categories_req = test::TestRequest::get()
            .uri("/categories")
            .header("Authorization", format!("Bearer {}", token))
            .to_request();

        let res: GetCategoriesResponse =
            test::read_response_json(&mut app, get_categories_req).await;
        assert!(res.categories.len() - 1 == category_count);

        let rename_category_req = test::TestRequest::put()
            .uri(&format!("/categories/{}", &category_id))
            .header("Authorization", format!("Bearer {}", token))
            .set_json(&RenameCategoryRequest {
                name: format!("new_category_name_{}", random_string(24)),
            })
            .to_request();

        let res = test::call_service(&mut app, rename_category_req).await;
        assert!(
            res.status().as_u16() == 204,
            "expected 204, but got response {:?}",
            res
        );
    }

    // User register and login
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

        let email = format!("test_{}@test.test", random_string(12));
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
                categories: vec![category_id],
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

        // Image search
        let search_images_req = test::TestRequest::get()
            .uri("/images")
            .header("Authorization", format!("Bearer {}", token))
            .to_request();
        let search_images_res = test::call_service(&mut app, search_images_req).await;
        assert!(
            search_images_res.status() == 200,
            "got {}, {:?}",
            search_images_res.status().as_u16(),
            test::read_body(search_images_res).await
        );
        let search_images_res_data: SearchImagesResponse =
            test::read_body_json(search_images_res).await;

        assert!(search_images_res_data.images.len() >= 1);

        // Finally image download
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
