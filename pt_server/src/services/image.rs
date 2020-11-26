use super::Service;
use crate::{
    config::Config, db::category::Category, db::category::CategoryExt, db::image::Image,
    db::image::NewImage, db::rating::Rating, model::image::*,
};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use async_trait::async_trait;
use futures::{future::join_all, StreamExt, TryStreamExt};
use regex::Regex;
use slog::{error, Logger};
use sqlx::PgPool;
use std::{ffi::OsString, path::Path};
use time::OffsetDateTime;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

pub const CATEGORY_NAME_PATTERN: &str = "[A-Za-z]+";

#[async_trait(?Send)]
pub trait ImageService: Service {
    async fn create_image(
        &self,
        app_user_id: Uuid,
        image: NewImage,
        categories: &[Uuid],
    ) -> Result<Uuid, CreateImageError>;
    async fn save_image(&self, id: Uuid, payload: Multipart) -> Result<(), UploadImageError>;
    async fn get_image(&self, id: Uuid) -> Result<NamedFile, std::io::Error>;
    async fn get_image_info(&self, id: Uuid) -> Result<(Image, Vec<Category>), GetImageInfoError>;
    async fn search_images(
        &self,
        search: Option<&str>,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<(Image, Vec<Category>)>, SearchImagesError>;

    async fn rate_image(
        &self,
        image_id: Uuid,
        app_user_id: Uuid,
        rating: u32,
    ) -> Result<(), RateImageError>;
    async fn get_image_ratings(&self, image_id: Uuid) -> Result<Vec<Rating>, GetImageRatingsError>;

    async fn get_categories(&self) -> Result<Vec<CategoryExt>, GetCategoriesError>;
    async fn create_category(&self, name: &str) -> Result<Uuid, CreateCategoryError>;
    async fn rename_category(&self, id: Uuid, name: &str) -> Result<(), RenameCategoryError>;
    async fn delete_category(&self, id: Uuid) -> Result<(), DeleteCategoryError>;
}
dyn_clone::clone_trait_object!(ImageService);

#[derive(Debug, Clone)]
pub struct DefaultImageService {
    pool: PgPool,
    logger: Logger,
    #[allow(dead_code)]
    config: Config,
}

impl DefaultImageService {
    pub fn new(config: &Config, logger: Logger, pool: PgPool) -> Self {
        Self {
            logger,
            pool,
            config: config.clone(),
        }
    }
}

#[async_trait(?Send)]
impl ImageService for DefaultImageService {
    async fn create_image(
        &self,
        app_user_id: Uuid,
        image: NewImage,
        categories: &[Uuid],
    ) -> Result<Uuid, CreateImageError> {
        let db_categories =
            futures::future::join_all(categories.iter().map(|id| Category::by_id(*id, &self.pool)))
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    error!(&self.logger, "unexpected database error";
                        "error" => e.to_string()
                    );
                    CreateImageError::Unexpected
                })?
                .into_iter()
                .zip(categories)
                .map(|(o, id)| o.ok_or(CreateImageError::CategoryNotFound(*id)))
                .collect::<Result<Vec<_>, _>>()?;

        let image_id = Image::new(app_user_id, image, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                CreateImageError::Unexpected
            })?;

        for category in db_categories {
            category
                .add_image(image_id, &self.pool)
                .await
                .map_err(|e| {
                    error!(&self.logger, "unexpected database error";
                        "error" => e.to_string()
                    );
                    CreateImageError::Unexpected
                })?;
        }

        Ok(image_id)
    }

    async fn save_image(&self, id: Uuid, mut payload: Multipart) -> Result<(), UploadImageError> {
        let img = Image::by_id(id, &self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            UploadImageError::Unexpected
        })?;

        match img {
            Some(mut img) => {
                if img.upload_date.is_some() {
                    return Err(UploadImageError::AlreadyUploaded);
                }

                match payload.try_next().await {
                    Ok(f) => match f {
                        Some(mut field) => {
                            let filepath = self
                                .config
                                .image_storage_path
                                .join(id.to_hyphenated().to_string())
                                .with_extension(
                                    field
                                        .content_disposition()
                                        .and_then(|c| {
                                            c.get_filename().and_then(|c| {
                                                Path::new(c).extension().map(|e| e.to_owned())
                                            })
                                        })
                                        .unwrap_or_else(|| OsString::from("png")),
                                );

                            fs::create_dir_all(&self.config.image_storage_path)
                                .await
                                .map_err(|e| {
                                    error!(&self.logger, "error creating directory for images";
                                    "error" => e.to_string()
                                    );
                                    UploadImageError::Unexpected
                                })?;

                            let mut file = File::create(filepath).await.map_err(|e| {
                                error!(&self.logger, "error saving the file";
                                    "error" => e.to_string()
                                );
                                UploadImageError::Unexpected
                            })?;

                            while let Some(chunk) = field.next().await {
                                let data = chunk.map_err(|e| {
                                    error!(&self.logger, "error saving the file";
                                        "error" => e.to_string()
                                    );
                                    UploadImageError::Unexpected
                                })?;

                                file.write_all(&data).await.map_err(|e| {
                                    error!(&self.logger, "error saving the file";
                                        "error" => e.to_string()
                                    );
                                    UploadImageError::Unexpected
                                })?;
                            }

                            img.upload_date = Some(OffsetDateTime::now_utc());

                            img.save(&self.pool).await.map_err(|e| {
                                error!(&self.logger, "unexpected database error";
                                    "error" => e.to_string()
                                );
                                UploadImageError::Unexpected
                            })?;
                            Ok(())
                        }
                        None => Err(UploadImageError::ExpectedFile),
                    },

                    Err(err) => {
                        error!(&self.logger, "unexpected upload error";
                            "error" => err.to_string()
                        );
                        Err(UploadImageError::Unexpected)
                    }
                }
            }
            None => Err(UploadImageError::InvalidId),
        }
    }

    async fn get_image(&self, id: Uuid) -> Result<NamedFile, std::io::Error> {
        NamedFile::open(
            &Path::new(&self.config.image_storage_path)
                .join(&id.to_hyphenated().to_string())
                .with_extension("png"),
        )
    }

    async fn search_images(
        &self,
        search: Option<&str>,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<(Image, Vec<Category>)>, SearchImagesError> {
        let images = Image::search(
            search,
            offset.map(|v| v as _),
            limit.map(|v| v as _),
            &self.pool,
        )
        .await
        .map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            SearchImagesError::Unexpected
        })?;

        let categories = join_all(images.iter().map(|i| i.categories(&self.pool)))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                SearchImagesError::Unexpected
            })?;

        Ok(images
            .into_iter()
            .zip(categories)
            // Filter only uploaded images
            .filter(|(i, _)| {
                Path::new(&self.config.image_storage_path)
                    .join(&i.id.to_hyphenated().to_string())
                    .with_extension("png")
                    .exists()
            })
            .collect())
    }

    async fn rate_image(
        &self,
        image_id: Uuid,
        app_user_id: Uuid,
        rating: u32,
    ) -> Result<(), RateImageError> {
        let image = Image::by_id(image_id, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                RateImageError::Unexpected
            })?
            .ok_or(RateImageError::ImageNotFound)?;

        if image.app_user_id == app_user_id {
            return Err(RateImageError::OwnImage);
        }

        if rating > 5 || rating == 0 {
            return Err(RateImageError::InvalidRating);
        }

        image
            .rate(app_user_id, rating as _, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                RateImageError::Unexpected
            })
    }

    async fn get_image_ratings(&self, image_id: Uuid) -> Result<Vec<Rating>, GetImageRatingsError> {
        let image = Image::by_id(image_id, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                GetImageRatingsError::Unexpected
            })?
            .ok_or(GetImageRatingsError::ImageNotFound)?;

        image.ratings(&self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            GetImageRatingsError::Unexpected
        })
    }

    async fn get_categories(&self) -> Result<Vec<CategoryExt>, GetCategoriesError> {
        CategoryExt::all(&self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            GetCategoriesError::Unexpected
        })
    }

    async fn create_category(&self, name: &str) -> Result<Uuid, CreateCategoryError> {
        let re = Regex::new(CATEGORY_NAME_PATTERN).unwrap();

        if !re.is_match(name) {
            return Err(CreateCategoryError::InvalidName(
                CATEGORY_NAME_PATTERN.into(),
            ));
        }

        let categories = Category::all(&self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            CreateCategoryError::Unexpected
        })?;

        let lc_name = name.to_lowercase();

        if categories
            .iter()
            .any(|c| c.category_name.to_lowercase() == lc_name)
        {
            return Err(CreateCategoryError::AlreadyExists);
        }

        Category::new(name, &self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            CreateCategoryError::Unexpected
        })
    }

    async fn rename_category(&self, id: Uuid, name: &str) -> Result<(), RenameCategoryError> {
        let re = Regex::new(CATEGORY_NAME_PATTERN).unwrap();

        if !re.is_match(name) {
            return Err(RenameCategoryError::InvalidName(
                CATEGORY_NAME_PATTERN.into(),
            ));
        }

        let mut category = Category::by_id(id, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                RenameCategoryError::Unexpected
            })?
            .ok_or(RenameCategoryError::CategoryNotFound)?;

        let categories = Category::all(&self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            RenameCategoryError::Unexpected
        })?;

        let lc_name = name.to_lowercase();

        if categories
            .iter()
            .any(|c| c.category_name.to_lowercase() == lc_name)
        {
            return Err(RenameCategoryError::AlreadyExists);
        }

        category.category_name = name.into();

        category.save(&self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            RenameCategoryError::Unexpected
        })
    }

    async fn delete_category(&self, id: Uuid) -> Result<(), DeleteCategoryError> {
        let category = Category::by_id(id, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                DeleteCategoryError::Unexpected
            })?
            .ok_or(DeleteCategoryError::CategoryNotFound)?;

        category.delete(&self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            DeleteCategoryError::Unexpected
        })
    }

    async fn get_image_info(&self, id: Uuid) -> Result<(Image, Vec<Category>), GetImageInfoError> {
        let image = Image::by_id(id, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                GetImageInfoError::Unexpected
            })?
            .ok_or(GetImageInfoError::NotFound)?;

        let categories = image.categories(&self.pool).await.map_err(|e| {
            error!(&self.logger, "unexpected database error";
                "error" => e.to_string()
            );
            GetImageInfoError::Unexpected
        })?;

        Ok((image, categories))
    }
}
