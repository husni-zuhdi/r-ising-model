use axum::extract::{MatchedPath, State};
use axum::http::{Request, StatusCode};
use axum::response::Html;
use axum::routing::{get, get_service};
use axum::Router;
use axum::{body::Bytes, http::HeaderMap, response::Response};
use std::fs;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::timeout::TimeoutLayer;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{info, info_span, Span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use web::config::Config;

// Application State
#[derive(Clone)]
struct AppState {
    config: Config,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    app().await;
    Ok(())
}

/// Run the axum web application
async fn app() {
    // Setup Config
    let config = Config::from_envar().await;
    let endpoint = format!("{}:{}", &config.svc_endpoint, &config.svc_port);

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Init app state
    info!("Starting HTTP Server at http://{}", endpoint);
    let app = main_route(AppState { config });

    // Start Axum Application
    let listener = tokio::net::TcpListener::bind(endpoint).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

/// Build Axum router
fn main_route(app_state: AppState) -> Router {
    let dist_path = app_state.config.dist_path.clone();
    Router::new()
        .route("/", get(get_index))
        .nest_service(
            "/assets",
            get_service(ServeDir::new(format!("{dist_path}/assets"))),
        )
        .nest_service(
            "/favicon.ico",
            get_service(ServeFile::new(format!("{dist_path}/favicon.ico"))),
        )
        .nest_service(
            "/gui_bg.wasm",
            get_service(ServeFile::new(format!("{dist_path}/gui_bg.wasm"))),
        )
        .nest_service(
            "/gui.js",
            get_service(ServeFile::new(format!("{dist_path}/gui.js"))),
        )
        .nest_service(
            "/index.css",
            get_service(ServeFile::new(format!("{dist_path}/index.css"))),
        )
        .nest_service(
            "/sw.js",
            get_service(ServeFile::new(format!("{dist_path}/sw.js"))),
        )
        .layer((
            ServiceBuilder::new().layer(CompressionLayer::new()),
            // TODO: explore more about TraceLayer
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);

                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        some_other_field = tracing::field::Empty,
                    )
                })
                .on_request(|request: &Request<_>, span: &Span| {
                    // You can use `_span.record("some_other_field", value)` in one of these
                    // closures to attach a value to the initially empty field in the info_span
                    // created above.
                    span.record("method", tracing::field::display(request.method()));
                    info!("started {} {}", request.method(), request.uri().path())
                })
                .on_response(|response: &Response, latency: Duration, span: &Span| {
                    span.record("status_code", tracing::field::display(response.status()));
                    info!("ended {} in {}ms", response.status(), latency.as_millis())
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    // ...
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {
                        // ...
                    },
                )
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        // ...
                    },
                ),
            // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
            // requests don't hang forever.
            TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(10)),
        ))
        .with_state(app_state)
        .fallback_service(get(get_not_found))
}

async fn get_index(State(app_state): State<AppState>) -> Html<String> {
    let dist_path = format!("{}/index.html", app_state.config.dist_path);
    let index = fs::read_to_string(&dist_path).unwrap_or("404 - Not Found".to_string());
    Html(index)
}

async fn get_not_found() -> Html<String> {
    Html("404 - Not Found".to_string())
}

// Handle shutdonw signal gracefully
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
