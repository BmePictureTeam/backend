use dyn_clone::DynClone;

pub mod auth;
pub mod image;

pub trait Service: Send + Sync + DynClone {}
impl<S> Service for S where S: Send + Sync + DynClone {}

pub use auth::{AuthService, DefaultAuthService};
pub use image::{ImageService, DefaultImageService};
