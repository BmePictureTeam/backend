use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage};
use futures::{
    future::{ok, Ready},
    Future,
};
use slog::info;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use time::OffsetDateTime;

pub struct Logger(slog::Logger);

impl Logger {
    pub fn new(logger: slog::Logger) -> Self {
        Self(logger)
    }
}

impl<S, B> Transform<S> for Logger
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
    logger: slog::Logger,
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
        let method = req.method().to_string();
        let log = self.logger.clone();

        req.extensions_mut().insert(String::new());

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

            let time_ms =
                (OffsetDateTime::now_utc() - start).whole_nanoseconds() as f32 / 1_000_000f32;

            info!(log, "request";
                "method" => method,
                "path" => path,
                "status" => res.status().as_u16(),
                "time" => format!("{}ms", time_ms),
                "userAgent" => agent,
                "address" => address,
            );

            Ok(res)
        })
    }
}
