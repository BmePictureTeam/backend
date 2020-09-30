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

SQLx relies on the database schema for type-safe SQL queries in code. A running database is required on the first compilation or whenever a migration is added. The connection string should be set in the `PT_DATABASE_URL` environment variable.

##### Migrations

Migrations can be found in [migrations](migrations) and can be applied with a utility: `cargo run --bin pt_util -- migrate`, or any other PostgreSQL utility, as they're plain SQL.

### Running Tests

Simply `cargo test`.

### Running The Server

Simply `cargo run --bin pt_server` or `cargo run --bin pt_server --release`.

### Building For Release

Use `cargo build --bin pt_server --release`, the built binary will be in `target/release/pt_server`.
