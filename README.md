# PictureTeam Server

The backend of Picture Team for Mobil&Fullstack Témalabor (currently only a hello-world).

- [PictureTeam Server](#pictureteam-server)
    - [Dependencies](#dependencies)
    - [Configuration](#configuration)
    - [Running Tests](#running-tests)
    - [Running The Server](#running-the-server)
    - [Building For Release](#building-for-release)

### Dependencies

- [A Rust toolchain](https://www.rust-lang.org/tools/install)
- OpenSSL library and headers, required by SQLx (`openssl-devel` on Fedora)

### Configuration

Configuration is done via environment variables prefixed with `PT_`, available options can be found in `src/config.rs`.

### Running Tests

Simply `cargo test`

### Running The Server

Simply `cargo run` or `cargo run --release`.

### Building For Release

Use `cargo build --release`, the built binary will be in `target/release/pt_server`.
