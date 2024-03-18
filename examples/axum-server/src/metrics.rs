use std::sync::Arc;

use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
    RequestExt,
};
use measured::{
    label::{self, LabelValue, StaticLabelSet},
    metric::histogram::Thresholds,
    text::TextEncoder,
    CounterVec, FixedCardinalityLabel, HistogramVec, LabelGroup, MetricGroup,
};
use tokio::{sync::Mutex, time::Instant};

use crate::AppState;

pub struct AppMetricsEncoder {
    encoder: Mutex<TextEncoder>,
    pub metrics: AppMetrics,
}

#[derive(MetricGroup)]
pub struct AppMetrics {
    pub http_requests: CounterVec<HttpRequestsSet>,
    pub http_responses: CounterVec<HttpResponsesSet>,
    pub http_request_duration: HistogramVec<HttpRequestsSet, 6>,
}

impl AppMetricsEncoder {
    pub fn new(metrics: AppMetrics) -> Self {
        Self {
            encoder: Mutex::default(),
            metrics,
        }
    }
}

impl AppMetrics {
    pub fn new(paths: lasso::RodeoReader) -> Self {
        let path = Arc::new(paths);
        Self {
            http_requests: CounterVec::new_sparse(HttpRequestsSet {
                method: StaticLabelSet::new(),
                path: path.clone(),
            }),
            http_responses: CounterVec::new_sparse(HttpResponsesSet {
                method: StaticLabelSet::new(),
                status: StaticLabelSet::new(),
                path: path.clone(),
            }),
            http_request_duration: HistogramVec::new_sparse_metric_vec(
                HttpRequestsSet {
                    method: StaticLabelSet::new(),
                    path: path.clone(),
                },
                Thresholds::exponential_buckets(0.1, 2.0),
            ),
        }
    }
}

pub async fn middleware(s: State<AppState>, mut request: Request, next: Next) -> Response {
    let AppMetrics {
        http_requests,
        http_responses,
        http_request_duration,
        ..
    } = &s.0.metrics.metrics;

    let path = request.extract_parts::<MatchedPath>().await.unwrap();
    let path = path.as_str();
    let method = request.method().clone().into();

    // record new request
    http_requests.inc(HttpRequests { path, method });

    let start = Instant::now();

    let response = next.run(request).await;

    // record http request duration
    let duration = start.elapsed();
    http_request_duration.observe(HttpRequests { path, method }, duration.as_secs_f64());

    // record http status response
    http_responses.inc(HttpResponses {
        path,
        method,
        status: StatusCode(response.status()),
    });

    response
}

pub async fn handler(s: State<AppState>) -> Response {
    let AppMetricsEncoder { encoder, metrics } = &*s.0.metrics;

    let mut encoder = encoder.lock().await;
    metrics.collect_group_into(&mut *encoder).unwrap();
    Response::new(encoder.finish().into())
}

#[derive(LabelGroup)]
#[label(set = HttpRequestsSet)]
pub struct HttpRequests<'a> {
    #[label(fixed_with = Arc<lasso::RodeoReader>)]
    path: &'a str,
    method: Method,
}

#[derive(LabelGroup)]
#[label(set = HttpResponsesSet)]
pub struct HttpResponses<'a> {
    #[label(fixed_with = Arc<lasso::RodeoReader>)]
    path: &'a str,
    method: Method,
    status: StatusCode,
}

// Some wrappers for http types to turn into metric label values

#[derive(Clone, Copy, FixedCardinalityLabel)]
enum Method {
    Get,
    Post,
    Other,
}

impl From<axum::http::Method> for Method {
    fn from(value: axum::http::Method) -> Self {
        if value == axum::http::Method::GET {
            Method::Get
        } else if value == axum::http::Method::POST {
            Method::Post
        } else {
            Method::Other
        }
    }
}

struct StatusCode(axum::http::StatusCode);

impl LabelValue for StatusCode {
    fn visit<V: label::LabelVisitor>(&self, v: V) -> V::Output {
        v.write_int(self.0.as_u16() as i64)
    }
}

impl FixedCardinalityLabel for StatusCode {
    fn cardinality() -> usize {
        (100..1000).len()
    }

    fn encode(&self) -> usize {
        self.0.as_u16() as usize - 100
    }

    fn decode(value: usize) -> Self {
        Self(axum::http::StatusCode::from_u16(u16::try_from(value).unwrap() + 100).unwrap())
    }
}
