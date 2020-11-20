use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;
use serde::{de::Error, Deserializer, Deserialize, Serialize, Serializer};
use time::OffsetDateTime;

pub const EMAIL_REGEX: &str = r#"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$"#;

pub fn validate_email(email: &str) -> bool {
    Regex::new(EMAIL_REGEX).unwrap().is_match(email)
}

#[test]
fn test_validate_email() {
    assert!(!validate_email("asdasd"));
    assert!(validate_email("asd@gmail.com"));
}

pub fn random_string(char_count: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(char_count)
        .collect()
}

pub fn serialize_rfc3339<S>(date: &OffsetDateTime, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    date.format(time::Format::Rfc3339).serialize(ser)
}

pub fn deserialize_rfc3339<'de, D>(de: D) -> Result<OffsetDateTime, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(de)?;
    Ok(OffsetDateTime::parse(s, time::Format::Rfc3339)
        .map_err(|e| D::Error::custom(&format!("invalid date: {}", e)))?)
}
