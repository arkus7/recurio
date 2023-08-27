use std::net::SocketAddr;

use axum::Router;
use tower_http::trace::TraceLayer;

use recurio::{configuration, startup::Application, telemetry};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    telemetry::init_subscriber("info".into());

    let configuration = configuration::get_configuration().expect("Failed to read configuration");

    let application = Application::build(configuration).await?;

    // build our application with a single route
    let app: Router = application.router().layer(
        TraceLayer::new_for_http()
            .make_span_with(telemetry::trace_layer_make_span_with)
            .on_request(telemetry::trace_layer_on_request)
            .on_response(telemetry::trace_layer_on_response),
    );

    // run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();

    Ok(())
}
