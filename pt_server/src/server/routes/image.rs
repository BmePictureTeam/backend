use crate::{
    config::Config,
    db::image::NewImage,
    model::error::GenericError,
    model::image::{
        CreateImageError, CreateImageRequest, CreateImageResponse, GetImageRatingResponse,
        GetImageRatingsError, Image, RateImageError, RateImageRequest, SearchImagesError,
        SearchImagesQuery, SearchImagesResponse, UploadImageError,
    },
    server::extractors::SessionToken,
    services::image::ImageService,
};
use actix_multipart::Multipart;
use actix_web::{
    get,
    http::StatusCode,
    post, put,
    web::{self, ServiceConfig},
    HttpRequest, HttpResponse,
};
use aide::openapi::v3::macros::api::define;
use aide::openapi::v3::macros::api;
use slog::{error, Logger};
use std::io;
use uuid::Uuid;

const TAG_NAME: &str = "images";

define::tag! {
    name(TAG_NAME),
    description("Operations with images"),
    display_name("Images")
}

#[api]
#[post("/images")]
#[tag(TAG_NAME)]
#[response(200, CreateImageResponse)]
#[response(400, GenericError)]
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
            &req.categories,
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
            CreateImageError::CategoryNotFound(_) => {
                HttpResponse::BadRequest().json(GenericError {
                    message: err.to_string(),
                })
            }
        },
    }
}

#[api]
#[get("/images")]
#[tag(TAG_NAME)]
#[response(200, SearchImagesResponse)]
async fn search_images(
    _token: SessionToken,
    req: web::Query<SearchImagesQuery>,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    match image_service
        .search_images(req.search.as_deref(), req.offset, req.limit)
        .await
    {
        Ok(images) => HttpResponse::Ok().json(SearchImagesResponse {
            images: images
                .into_iter()
                .map(|(i, c)| Image {
                    title: i.title,
                    description: i.description,
                    categories: c.into_iter().map(|c| c.id).collect(),
                })
                .collect(),
        }),
        Err(err) => match err {
            SearchImagesError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
        },
    }
}

#[api]
#[post("/images/{image_id}")]
#[tag(TAG_NAME)]
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
#[tag(TAG_NAME)]
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

#[api]
#[put("/images/{image_id}/rating")]
#[tag(TAG_NAME)]
#[response(204)]
#[response(
    status(404),
    type(GenericError),
    description("the image was not found")
)]
#[response(
    status(403),
    type(GenericError),
    description("own image cannot be rated")
)]
async fn rate_image(
    token: SessionToken,
    web::Path((image_id,)): web::Path<(Uuid,)>,
    req: web::Json<RateImageRequest>,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    match image_service
        .rate_image(image_id, token.user_info().id, req.rating)
        .await
    {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => match err {
            RateImageError::ImageNotFound => HttpResponse::NotFound().json(GenericError {
                message: err.to_string(),
            }),
            RateImageError::OwnImage => HttpResponse::Forbidden().json(GenericError {
                message: err.to_string(),
            }),
            RateImageError::InvalidRating => HttpResponse::BadRequest().json(GenericError {
                message: err.to_string(),
            }),
            RateImageError::Unexpected => HttpResponse::InternalServerError().json(GenericError {
                message: err.to_string(),
            }),
        },
    }
}

#[api]
#[get("/images/{image_id}/rating")]
#[tag(TAG_NAME)]
#[response(200, GetImageRatingResponse)]
#[response(404)]
async fn get_image_rating(
    _token: SessionToken,
    web::Path((image_id,)): web::Path<(Uuid,)>,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    match image_service.get_image_ratings(image_id).await {
        Ok(ratings) => HttpResponse::Ok().json(GetImageRatingResponse {
            average: if ratings.len() > 0 {
                ratings.iter().fold(0, |c, r| c + r.rating) as f32 / ratings.len() as f32
            } else {
                0.0
            },
            rating_count: ratings.len() as _,
        }),
        Err(err) => match err {
            GetImageRatingsError::ImageNotFound => HttpResponse::NotFound().finish(),
            GetImageRatingsError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
        },
    }
}

pub fn configure_routes(_config: &Config) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut ServiceConfig| {
        app.service(create_image);
        app.service(upload_image);
        app.service(download_image);
        app.service(search_images);
        app.service(rate_image);
        app.service(get_image_rating);
    }
}
