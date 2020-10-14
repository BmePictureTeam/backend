use std::io;

use crate::{
    config::Config,
    db::image::NewImage,
    model::error::GenericError,
    model::image::{CreateImageError, CreateImageRequest, CreateImageResponse, UploadImageError},
    server::extractors::SessionToken,
    services::image::ImageService,
};
use actix_multipart::Multipart;
use actix_web::{
    get,
    http::StatusCode,
    post,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};
use aide::openapi::v3::macros::api;
use slog::{error, Logger};
use uuid::Uuid;

#[api]
#[post("/images")]
#[response(200, CreateImageResponse)]
async fn create_image(
    token: SessionToken,
    req: web::Json<CreateImageRequest>,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    match image_service
        .create_image(
            token.user_info().id,
            NewImage {
                description: req.description.clone(),
                title: req.title.clone(),
            },
        )
        .await
    {
        Ok(id) => HttpResponse::Ok().json(CreateImageResponse { id }),
        Err(err) => match err {
            CreateImageError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError {
                    message: err.to_string(),
                })
            }
        },
    }
}

#[api]
#[post("/images/{image_id}")]
#[response(204)]
#[response(400, GenericError)]
async fn upload_image(
    _token: SessionToken,
    web::Path((image_id,)): web::Path<(Uuid,)>,
    payload: Multipart,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    match image_service.save_image(image_id, payload).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => match err {
            UploadImageError::InvalidId
            | UploadImageError::AlreadyUploaded
            | UploadImageError::ExpectedFile
            | UploadImageError::TimeOut(_) => HttpResponse::BadRequest().json(GenericError {
                message: err.to_string(),
            }),
            UploadImageError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError {
                    message: err.to_string(),
                })
            }
        },
    }
}

#[api]
#[get("/images/{image_id}")]
#[response(status(200), content_type("application/octet-stream"))]
#[response(404)]
async fn download_image(
    _token: SessionToken,
    web::Path((image_id,)): web::Path<(Uuid,)>,
    req: HttpRequest,
    logger: web::Data<Logger>,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    match image_service.get_image(image_id).await {
        Ok(f) => match f.set_status_code(StatusCode::OK).into_response(&req) {
            Ok(res) => res,
            Err(err) => {
                error!(logger, "error serving the image";
                    "error" => err.to_string()
                );
                HttpResponse::InternalServerError().json(GenericError::default())
            }
        },
        Err(err) => match err.kind() {
            io::ErrorKind::NotFound => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().json(GenericError::default()),
        },
    }
}

pub fn configure_routes(_config: &Config) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut ServiceConfig| {
        app.service(create_image);
        app.service(upload_image);
        app.service(download_image);
    }
}
