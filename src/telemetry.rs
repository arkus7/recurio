use axum::{
    body::BoxBody,
    extract::{ConnectInfo, MatchedPath},
    response::Response,
};
use hyper::{Body, Request};
use std::{net::SocketAddr, time::Duration};

use tracing::Span;

use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use uuid::Uuid;

pub fn init_subscriber(env_filter: String) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();
}

pub fn trace_layer_make_span_with(request: &Request<Body>) -> Span {
    let matched_path = request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .unwrap_or_default();

    let user_agent = request
        .headers()
        .get("User-Agent")
        .map(|h| h.to_str().unwrap_or(""))
        .unwrap_or("");

    let conn_info = request.extensions().get::<ConnectInfo<SocketAddr>>();

    tracing::info_span!("HTTP request",
        http.method = %request.method(),
        http.route = %matched_path,
        http.flavor = ?request.version(),
        http.scheme = ?request.uri().scheme(),
        http.user_agent = %user_agent,
        http.target = %request.uri(),
        otel.name = %format!("HTTP {} {}", request.method(), matched_path),
        otel.kind = "server",
        // This is not particularly robust, but suitable for a demo
        // You'll need to change this if you deploy behind a proxy
        // (eg the `X-forwarded-for` header)
        http.client_ip = conn_info.map(|connect_info|
                tracing::field::display(connect_info.ip().to_string()),
            ).unwrap_or_else(||
                tracing::field::display(String::from("<unknown>"))
            ),
        request_id = %Uuid::new_v4(),
        // Fields must be defined to be used, define them as empty if they populate later

        otel.status_code = tracing::field::Empty,
        latency = tracing::field::Empty,
        res.headers = tracing::field::Empty,
        req.headers = tracing::field::Empty,
    )
}

pub fn trace_layer_on_request(request: &Request<Body>, span: &Span) {
    tracing::trace!("Got request");
    span.record("req.headers", tracing::field::debug(request.headers()));
}

pub fn trace_layer_on_response(response: &Response<BoxBody>, latency: Duration, span: &Span) {
    span.record(
        "latency",
        tracing::field::display(format!("{}ms", latency.as_millis())),
    );
    span.record(
        "otel.status_code",
        tracing::field::display(response.status()),
    );
    span.record("res.headers", tracing::field::debug(response.headers()));
    tracing::trace!("Responded");
}
