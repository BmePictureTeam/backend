use crate::config::Config;
use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::{
    future::{ok, Ready},
    Future,
};
use slog::{info, o, Drain, Logger};
use std::{
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
};
use time::{Duration, OffsetDateTime};

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

pub struct LoggerMw(Logger);

impl LoggerMw {
    pub fn new(logger: Logger) -> Self {
        Self(logger)
    }
}

impl<S, B> Transform<S> for LoggerMw
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggerMiddleware {
            service,
            logger: self.0.clone(),
        })
    }
}

#[doc(hidden)]
pub struct LoggerMiddleware<S> {
    service: S,
    logger: Logger,
}

#[allow(clippy::type_complexity)]
impl<S, B> Service for LoggerMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        let start = OffsetDateTime::now_utc();
        let log = self.logger.clone();

        let address = req
            .connection_info()
            .realip_remote_addr()
            .map(|s| s.to_string());

        let agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()));

        let next = self.service.call(req);

        Box::pin(async move {
            let res = next.await?;

            let status = res.status();
            let latency: Duration = OffsetDateTime::now_utc() - start;

            info!(log, "request";
                "path" => path,
                "status" => status.to_string(),
                "time" => latency.whole_milliseconds(),
                "userAgent" => agent,
                "address" => address,
            );

            Ok(res)
        })
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
