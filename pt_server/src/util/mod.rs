use rand::{distributions::Alphanumeric, thread_rng, Rng};
use regex::Regex;

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
