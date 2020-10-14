use super::Service;
use crate::{
    config::Config, db::image::Image, db::image::NewImage, model::image::CreateImageError,
    model::image::UploadImageError,
};
use actix_files::NamedFile;
use actix_multipart::Multipart;
use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use slog::{error, Logger};
use sqlx::PgPool;
use std::{ffi::OsString, path::Path};
use time::OffsetDateTime;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use uuid::Uuid;

#[async_trait(?Send)]
pub trait ImageService: Service {
    async fn create_image(
        &self,
        app_user_id: Uuid,
        image: NewImage,
    ) -> Result<Uuid, CreateImageError>;
    async fn save_image(&self, id: Uuid, payload: Multipart) -> Result<(), UploadImageError>;
    async fn get_image(&self, id: Uuid) -> Result<NamedFile, std::io::Error>;
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
    ) -> Result<Uuid, CreateImageError> {
        Image::new(app_user_id, image, &self.pool)
            .await
            .map_err(|e| {
                error!(&self.logger, "unexpected database error";
                    "error" => e.to_string()
                );
                CreateImageError::Unexpected
            })
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
                if img.image.upload_date.is_some() {
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

                            img.image.upload_date = Some(OffsetDateTime::now_utc());

                            img.image.save(&self.pool).await.map_err(|e| {
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
}
