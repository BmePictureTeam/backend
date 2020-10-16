use crate::config::Config;
use slog::{o, Drain, Logger};
use std::sync::Mutex;

pub fn create_logger(config: &Config) -> Logger {
    if config.log_json {
        slog::Logger::root(
            slog_async::Async::new(
                Mutex::new(slog_json::Json::default(std::io::stderr())).map(slog::Fuse),
            )
            .build()
            .fuse(),
            o!("version" => env!("CARGO_PKG_VERSION")),
        )
    } else {
        slog::Logger::root(
            slog_async::Async::new(
                slog_term::FullFormat::new(slog_term::TermDecorator::new().build())
                    .build()
                    .fuse(),
            )
            .build()
            .fuse(),
            o!("version" => env!("CARGO_PKG_VERSION")),
        )
    }
}

pub trait LoggerExt: private::Sealed {
    fn with_scope<S: ToString>(&self, scope: S) -> Self;
}

impl LoggerExt for Logger {
    fn with_scope<S: ToString>(&self, scope: S) -> Self {
        self.new(o!("scope" => scope.to_string()))
    }
}

mod private {
    use slog::Logger;

    pub trait Sealed {}
    impl Sealed for Logger {}
}
