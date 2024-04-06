use std::{sync::Arc, time::Duration};

use aide::{
    axum::{routing::get, ApiRouter},
    openapi::OpenApi,
};
use axum::{extract::Path, http::StatusCode, middleware};
use metrics::{AppMetrics, AppMetricsEncoder};
use tokio::net::TcpListener;

mod metrics;

#[derive(Clone)]
struct AppState {
    metrics: Arc<AppMetricsEncoder>,
}

#[tokio::main]
async fn main() {
    // We use the openapi features from `aide` in order to record the API routes we register.
    let mut api = OpenApi::default();

    let app = ApiRouter::new()
        .nest("/api/v1", api_v1())
        .api_route("/metrics", get(metrics::handler))
        .finish_api(&mut api);

    let paths = api
        .paths
        .unwrap()
        .iter()
        // this is a little awkward
        // aide replaces the `:id` with `{id}` so we need to undo that...
        .map(|(path, _)| path.replace('{', ":").replace('}', ""))
        .collect::<lasso::Rodeo>()
        .into_reader();

    // Using the routes captured in the OpenApi object, we build the app metrics
    let state = AppState {
        metrics: Arc::new(AppMetricsEncoder::new(AppMetrics::new(Arc::new(paths)))),
    };

    let app = app
        .layer(middleware::from_fn_with_state(
            state.clone(),
            metrics::middleware,
        ))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

fn api_v1() -> ApiRouter<AppState> {
    ApiRouter::new()
        .nest("/users", users())
        .nest("/products", products())
}

fn users() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(|| async { StatusCode::OK }))
        .api_route(
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

fn products() -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", get(|| async { StatusCode::OK }))
        .api_route(
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
        .api_route(
            "/:id/owner",
            get(|p: Path<String>| async move {
                if p.0 == "awesome" {
                    StatusCode::OK
                } else {
                    StatusCode::NOT_FOUND
                }
            }),
        )
        .api_route(
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
