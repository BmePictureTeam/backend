pub mod app_user;
pub mod auth;

pub trait Service: Send + Sync {}
impl<S> Service for S where S: Send + Sync {}

pub use app_user::{AppUserService, DefaultAppUserService};
pub use auth::{AuthService, DefaultAuthService};
