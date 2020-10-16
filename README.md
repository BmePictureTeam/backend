# PictureTeam Server

The backend of Picture Team for Mobil&Fullstack Témalabor (currently only a hello-world).

- [PictureTeam Server](#pictureteam-server)
    - [Dependencies](#dependencies)
    - [Configuration](#configuration)
      - [Database](#database)
        - [Migrations](#migrations)
    - [Running Tests](#running-tests)
    - [Running The Server](#running-the-server)
    - [Building For Release](#building-for-release)

### Dependencies

- [A Rust toolchain](https://www.rust-lang.org/tools/install)
- OpenSSL library and headers, required by SQLx (`openssl-devel` on Fedora)

### Configuration

Configuration is done via environment variables prefixed with `PT_`, available options can be found in `src/config.rs`.

#### Database

A PostgreSQL database is required for development and deployment.

SQLx relies on the database schema for type-safe SQL queries in code. A running database with the correct schema is required whenever the database-specific code is changed.

##### Migrations

Migrations can be found in [migrations](migrations) and should be applied with the [`sqlx-cli`](https://lib.rs/crates/sqlx-cli) utility: `sqlx mig run`.

### Running Tests

Run `cargo test -p pt_server --lib`.

The output should be something like:
```
    Finished test [unoptimized + debuginfo] target(s) in 0.09s
     Running target/debug/deps/pt_server-6da2475205f47403

running 2 tests
test util::test_validate_email ... ok
test tests::integration::whole_app ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Running The Server

Simply `cargo run --bin pt_server` or `cargo run --bin pt_server --release`.

### Building For Release

Use `cargo build --bin pt_server --release`, the built binary will be in `target/release/pt_server`.
