use crate::model::instrument::{Instrument, InstrumentFamily};
use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use aide::openapi::v3::macros::api;
use once_cell::sync::Lazy;
use rand::prelude::*;
use url::Url;

static INSTRUMENTS: Lazy<[Instrument; 4]> = Lazy::new(|| {
    [
    Instrument {
        id: 0,
        family: InstrumentFamily::Strings,
        image_url: Url::parse(
            "https://upload.wikimedia.org/wikipedia/commons/0/01/Gibson_Les_Paul_54_Custom.jpg",
        )
        .unwrap(),
        name: "Electric Guitar".into(),
    },
    Instrument {
        id: 0,
        family: InstrumentFamily::Strings,
        image_url: Url::parse(
            "https://upload.wikimedia.org/wikipedia/commons/thumb/4/4d/C.F._Martin_GRH_160_or_000-16RGT_cropped.png/800px-C.F._Martin_GRH_160_or_000-16RGT_cropped.png",
        )
        .unwrap(),
        name: "Acoustic Guitar".into(),
    },
    Instrument {
        id: 0,
        family: InstrumentFamily::Keyboard,
        image_url: Url::parse(
            "https://upload.wikimedia.org/wikipedia/commons/c/c8/Grand_piano_and_upright_piano.jpg",
        )
        .unwrap(),
        name: "Piano".into(),
    },
    Instrument {
        id: 0,
        family: InstrumentFamily::Percussion,
        image_url: Url::parse(
            "https://upload.wikimedia.org/wikipedia/commons/thumb/f/f2/OutsideBRX-15.JPG/1920px-OutsideBRX-15.JPG",
        )
        .unwrap(),
        name: "Drums".into(),
    },
]
});

fn random_instruments(count: usize) -> Vec<Instrument> {
    let mut rng = thread_rng();

    (0..count)
        .into_iter()
        .map(|i| {
            let mut instrument = INSTRUMENTS[rng.gen_range(0, INSTRUMENTS.len())].clone();
            instrument.id = i;
            instrument
        })
        .collect()
}

#[api]
#[serde(rename_all = "camelCase")]
struct InstrumentsQuery {
    /// The amount of instruments to retrieve.
    count: Option<usize>,
}

/// Default instruments response.
#[api]
#[serde(rename_all = "camelCase")]
struct InstrumentsResponse {
    instruments: Vec<Instrument>,
}

/// Returns a list of random instruments.
#[api]
#[get("/instruments")]
#[response(200, InstrumentsResponse)]
async fn get_instruments(query: web::Query<InstrumentsQuery>) -> impl Responder {
    HttpResponse::Ok().json(InstrumentsResponse {
        instruments: random_instruments(query.count.unwrap_or(10)),
    })
}

pub fn setup_routes(app: &mut ServiceConfig) {
    app.service(get_instruments);
}
