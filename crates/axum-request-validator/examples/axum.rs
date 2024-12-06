//! Basic axum middleware usage example.

use axum::{routing::get, Router};

/// Require that body in empty.
#[derive(Debug, Clone, Copy)]
struct EmptyValidator;

impl<Data: bytes::Buf + Send + Sync> http_request_validator::Validator<Data> for EmptyValidator {
    type Error = &'static str;

    async fn validate<'a>(
        &'a self,
        _parts: &'a axum::http::request::Parts,
        buffered_body: &'a Data,
    ) -> Result<(), Self::Error> {
        if buffered_body.has_remaining() {
            return Err("body not empty");
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route_layer(axum_request_validator::new(EmptyValidator));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
