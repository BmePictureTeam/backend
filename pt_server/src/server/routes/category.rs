use crate::{
    config::Config, model::error::GenericError, model::image::*, server::extractors::SessionToken,
    services::ImageService,
};
use actix_web::{
    get, post, put,
    web::{self, ServiceConfig},
    HttpResponse,
};
use aide::openapi::v3::macros::api;
use uuid::Uuid;

#[api]
#[get("/categories")]
#[response(200, GetCategoriesResponse)]
async fn get_categories(
    _token: SessionToken,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    match image_service.get_categories().await {
        Ok(categories) => HttpResponse::Ok().json(GetCategoriesResponse {
            categories: categories
                .into_iter()
                .map(|c| Category {
                    id: c.category.id,
                    name: c.category.category_name,
                    image_count: c.image_count as _,
                })
                .collect(),
        }),
        Err(err) => match err {
            GetCategoriesError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
        },
    }
}

#[api]
#[post("/categories")]
#[response(200, CreateCategoryResponse)]
async fn create_category(
    token: SessionToken,
    req: web::Json<CreateCategoryRequest>,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    if !token.user_info().admin {
        return HttpResponse::Forbidden().json(GenericError {
            message: CreateCategoryError::NotAllowed.to_string(),
        });
    }

    match image_service.create_category(&req.name).await {
        Ok(id) => HttpResponse::Ok().json(CreateCategoryResponse { id }),
        Err(err) => match err {
            CreateCategoryError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
            _ => unreachable!(),
        },
    }
}

#[api]
#[put("/categories/{category_id}")]
#[response(204)]
#[response(404)]
async fn rename_category(
    token: SessionToken,
    web::Path((category_id,)): web::Path<(Uuid,)>,
    req: web::Json<RenameCategoryRequest>,
    image_service: web::Data<Box<dyn ImageService>>,
) -> HttpResponse {
    if !token.user_info().admin {
        return HttpResponse::Forbidden().json(GenericError {
            message: RenameCategoryError::NotAllowed.to_string(),
        });
    }

    match image_service.rename_category(category_id, &req.name).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => match err {
            RenameCategoryError::Unexpected => {
                HttpResponse::InternalServerError().json(GenericError::default())
            }
            _ => unreachable!(),
        },
    }
}

pub fn configure_routes(_config: &Config) -> impl FnOnce(&mut ServiceConfig) {
    move |app: &mut ServiceConfig| {
        app.service(get_categories);
        app.service(create_category);
        app.service(rename_category);
    }
}
