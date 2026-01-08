use axum::routing::get_service;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::info;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    app().await;
    Ok(())
}

/// Run the axum web application
pub async fn app() {
    // Setup Config
    //let config = Config::from_envar().await;
    //let endpoint = format!("{}:{}", &config.svc_endpoint, &config.svc_port);
    let endpoint = "0.0.0.0:8080";

    // Initialize Tracing
    tracing_subscriber::fmt()
        //.with_max_level(config.log_level)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Init app state
    //let app_state = state_factory(config).await;
    info!("Starting HTTP Server at http://{}", endpoint);
    let app = main_route();

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(endpoint).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub fn main_route() -> Router {
    Router::new()
        .nest_service("/assets", get_service(ServeDir::new("./dist/assets")))
        .nest_service(
            "/favicon.ico",
            get_service(ServeFile::new("./dist/favicon.ico")),
        )
        .nest_service(
            "/gui_bg.wasm",
            get_service(ServeFile::new("./dist/gui_bg.wasm")),
        )
        .nest_service("/gui.js", get_service(ServeFile::new("./dist/gui.js")))
        .nest_service(
            "/index.css",
            get_service(ServeFile::new("./dist/index.css")),
        )
        .nest_service("/sw.js", get_service(ServeFile::new("./dist/sw.js")))
        .layer(ServiceBuilder::new().layer(CompressionLayer::new()))
        .fallback_service(ServeFile::new("./dist/index.html"))
}
