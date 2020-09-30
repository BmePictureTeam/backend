use aide::openapi::v3::macros::api;
use url::Url;

/// The family (category) of an instrument.
#[api::model]
#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone)]
pub enum InstrumentFamily {
    Strings,
    Keyboard,
    Woodwind,
    Brass,
    Percussion,
}

/// An instrument.
#[api::model]
#[derive(Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Instrument {
    /// Unique ID of the instrument.
    pub id: usize,

    /// Name of the instrument.
    pub name: String,

    /// The family (category) of the instrument.
    pub family: InstrumentFamily,

    /// External image of the instrument.
    #[schemars(with = "String")]
    pub image_url: Url,
}
