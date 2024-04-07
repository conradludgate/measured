use std::{sync::Arc, time::Duration};

use axum::{
    extract::{FromRef, Path},
    http::StatusCode,
    middleware,
    routing::get,
    Router,
};
use metrics::{AppMetrics, AppMetricsEncoder};
use tokio::net::TcpListener;

mod metrics;

#[derive(Clone)]
struct AppState {
    metrics: Arc<AppMetricsEncoder>,
}

impl FromRef<AppState> for Arc<AppMetricsEncoder> {
    fn from_ref(input: &AppState) -> Self {
        input.metrics.clone()
    }
}

#[tokio::main]
async fn main() {
    let state = AppState {
        metrics: Arc::new(AppMetricsEncoder::new(AppMetrics::new(Arc::new(
            lasso::ThreadedRodeo::new(),
        )))),
    };

    let app = Router::new()
        // add our API routes
        .nest("/api/v1", api_v1())
        // add the metrics exporter
        .route("/metrics", get(metrics::handler))
        // add the metrics middleware
        .layer(middleware::from_fn_with_state(
            state.clone(),
            metrics::middleware,
        ))
        // add the state
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn api_v1() -> Router<AppState> {
    Router::new()
        .nest("/users", users())
        .nest("/products", products())
}

fn users() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { StatusCode::OK }))
        .route(
            "/:id",
            get(|p: Path<String>| async move {
                if p.0 == "current" {
                    StatusCode::OK
                } else {
                    StatusCode::BAD_REQUEST
                }
            })
            .post(|| async { StatusCode::FORBIDDEN }),
        )
}

fn products() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { StatusCode::OK }))
        .route(
            "/:id",
            get(|p: Path<String>| async move {
                if p.0 == "awesome" {
                    StatusCode::OK
                } else {
                    StatusCode::NOT_FOUND
                }
            })
            .post(|| async { StatusCode::CREATED }),
        )
        .route(
            "/:id/owner",
            get(|p: Path<String>| async move {
                if p.0 == "awesome" {
                    StatusCode::OK
                } else {
                    StatusCode::NOT_FOUND
                }
            }),
        )
        .route(
            "/:id/purchase",
            get(|p: Path<String>| async move {
                if p.0 == "awesome" {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    StatusCode::OK
                } else {
                    StatusCode::NOT_FOUND
                }
            }),
        )
}
