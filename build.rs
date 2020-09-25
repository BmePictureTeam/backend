fn main() {
    // Remove the prefix for SQLx type checks,
    // as it relies on the hardcoded DATABASE_URL env var.
    if let Ok(url) = std::env::var("PT_DATABASE_URL") {
        println!("cargo:rustc-env=DATABASE_URL={}", url);
    } else {
        // Set a default DB url
        println!(
            "cargo:rustc-env=DATABASE_URL=postgresql://postgres:postgres@localhost:5432/postgres"
        );
    }
}
