use std::sync::Arc;

use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
    RequestExt,
};
use measured::{
    label::{self, LabelValue},
    metric::histogram::Thresholds,
    text::BufferedTextEncoder,
    CounterVec, FixedCardinalityLabel, HistogramVec, LabelGroup, MetricGroup,
};
use tokio::sync::Mutex;

/// Defines both the metrics and the metrics encoder.
/// Will be stored in the axum state.
pub struct AppMetricsEncoder {
    encoder: Mutex<BufferedTextEncoder>,
    pub metrics: AppMetrics,
}

/// The metrics we wish to export
#[derive(MetricGroup)]
#[metric(new(paths: Arc<lasso::ThreadedRodeo>))]
pub struct AppMetrics {
    /// total number of HTTP requests by path and method
    #[metric(label_set = HttpRequestsSet::new(paths.clone()))]
    pub http_requests: CounterVec<HttpRequestsSet>,

    /// total number of HTTP responses by path, method, and status
    #[metric(label_set = HttpResponsesSet::new(paths.clone()))]
    pub http_responses: CounterVec<HttpResponsesSet>,

    /// duration of HTTP handlers by path and method
    #[metric(
        label_set = HttpRequestsSet::new(paths),
        // starting at 0.1ms up to 6.5s
        metadata = Thresholds::exponential_buckets(0.0001, 2.0),
    )]
    pub http_request_duration: HistogramVec<HttpRequestsSet, 16>,
}

impl AppMetricsEncoder {
    pub fn new(metrics: AppMetrics) -> Self {
        Self {
            encoder: Mutex::default(),
            metrics,
        }
    }
}

/// Middleware wraps all HTTP requests to automatically report metrics
pub async fn middleware(
    s: State<Arc<AppMetricsEncoder>>,
    mut request: Request,
    next: Next,
) -> Response {
    let AppMetrics {
        http_requests,
        http_responses,
        http_request_duration,
        ..
    } = &s.0.metrics;

    // extract the 'matched path', which excludes any filled in patterns.
    // eg "/users/conradludgate" => "/users/:id"
    let path = request.extract_parts::<MatchedPath>().await;
    let path = match &path {
        Ok(path) => path.as_str(),
        Err(_) => "unknown",
    };
    let method = request.method().into();

    // record new request
    http_requests.inc(HttpRequests { path, method });

    let timer = http_request_duration.start_timer(HttpRequests { path, method });

    let response = next.run(request).await;

    // record http request duration
    timer.observe();

    // record http response with status
    http_responses.inc(HttpResponses {
        path,
        method,
        status: StatusCode(response.status()),
    });

    response
}

/// sample and export the metrics
pub async fn handler(s: State<Arc<AppMetricsEncoder>>) -> Response {
    let AppMetricsEncoder { encoder, metrics } = &*s.0;

    let mut encoder = encoder.lock().await;
    metrics.collect_group_into(&mut *encoder).unwrap();
    Response::new(encoder.finish().into())
}

#[derive(LabelGroup)]
#[label(set = HttpRequestsSet)]
pub struct HttpRequests<'a> {
    #[label(dynamic_with = Arc<lasso::ThreadedRodeo>)]
    path: &'a str,
    method: Method,
}

#[derive(LabelGroup)]
#[label(set = HttpResponsesSet)]
pub struct HttpResponses<'a> {
    #[label(dynamic_with = Arc<lasso::ThreadedRodeo>)]
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

impl From<&axum::http::Method> for Method {
    fn from(value: &axum::http::Method) -> Self {
        if value == axum::http::Method::GET {
            Method::Get
        } else if value == axum::http::Method::POST {
            Method::Post
        } else {
            Method::Other
        }
    }
}

#[derive(Clone, Copy)]
struct StatusCode(axum::http::StatusCode);

impl LabelValue for StatusCode {
    fn visit<V: label::LabelVisitor>(&self, v: V) -> V::Output {
        v.write_int(self.0.as_u16() as i64)
    }
}

impl FixedCardinalityLabel for StatusCode {
    fn cardinality() -> usize {
        // Status code values in the range 100-999 (inclusive) are supported by this type
        (100..1000).len()
    }

    fn encode(&self) -> usize {
        self.0.as_u16() as usize - 100
    }

    fn decode(value: usize) -> Self {
        Self(axum::http::StatusCode::from_u16(u16::try_from(value).unwrap() + 100).unwrap())
    }
}
