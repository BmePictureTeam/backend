use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

use crate::config::Config;

pub fn routes(app: &mut ServiceConfig) {
    app.service(hello_picture_team);
}

#[derive(Debug, Deserialize)]
struct HelloQuery {
    name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HelloResponse {
    pub message: String,
    pub server_config: Config,
}

#[get("/hello")]
async fn hello_picture_team(
    query: web::Query<HelloQuery>,
    config: web::Data<Config>,
) -> impl Responder {
    HttpResponse::Ok()
        .content_type("application/json")
        .json(HelloResponse {
            message: format!(
                "Hello from the Picture Team, {}!",
                query
                    .into_inner()
                    .name
                    .unwrap_or_else(|| "visitor".to_string())
            ),
            server_config: (&*config.into_inner()).clone(),
        })
}
