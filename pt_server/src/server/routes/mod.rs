use actix_web::web::ServiceConfig;

mod instruments;

pub fn setup_routes(app: &mut ServiceConfig) {
    instruments::setup_routes(app);
}
