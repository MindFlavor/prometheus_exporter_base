//! This crate simplifies the generation of valid Prometheus metrics. You should start by building
//! a [`PrometheusMetric`] and then adding as many [`PrometheusInstance`] as needed.
//!
//! [`PrometheusMetric`] specifies the [`counter name`], [`type`] and [`help`]. Each [`PrometheusInstance`]
//! specifies the [`value`] and the optional [`labels`] and [`timestamp`].
//!
//! [`counter name`]: prometheus_metric_builder/struct.PrometheusMetricBuilder.html#method.with_name
//! [`type`]: prometheus_metric_builder/struct.PrometheusMetricBuilder.html#method.with_metric_type
//! [`help`]: prometheus_metric_builder/struct.PrometheusMetricBuilder.html#method.with_help
//! [`counter name`]: struct.PrometheusMetricBuilder.html#method.with_name
//! [`value`]: struct.PrometheusInstance.html#method.with_value
//! [`labels`]: struct.PrometheusInstance.html#method.with_label
//! [`timestamp`]: struct.PrometheusInstance.html#method.with_timestamp
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use prometheus_exporter_base::prelude::*;
//!
//! let rendered_string = PrometheusMetric::build()
//!     .with_name("folder_size")
//!     .with_metric_type(MetricType::Counter)
//!     .with_help("Size of the folder")
//!     .build()
//!     .render_and_append_instance(
//!         &PrometheusInstance::new()
//!             .with_label("folder", "/var/log")
//!             .with_value(100)
//!             .with_current_timestamp()
//!             .expect("error getting the UNIX epoch"),
//!     )
//!     .render();
//! ```

extern crate failure;
extern crate serde_json;
use http::StatusCode;
use hyper::service::{make_service_fn, service_fn};
use hyper::Client;
use hyper::{body, Body, Request, Response, Server};
use log::{debug, error, info, trace, warn};
use serde::de::DeserializeOwned;
use std::sync::Arc;
mod prometheus_metric;
mod render_to_prometheus;
pub use prometheus_metric::PrometheusMetric;
pub mod prelude;
pub use render_to_prometheus::RenderToPrometheus;
mod metric_type;
mod prometheus_instance;
pub use metric_type::MetricType;
pub use prometheus_instance::{MissingValue, PrometheusInstance};
use std::future::Future;
use std::net::SocketAddr;
pub mod prometheus_metric_builder;

pub trait ToAssign {}
#[derive(Debug, Clone, Copy)]
pub struct Yes {}
#[derive(Debug, Clone, Copy)]
pub struct No {}
impl ToAssign for Yes {}
impl ToAssign for No {}

#[inline]
async fn extract_body(resp: hyper::client::ResponseFuture) -> Result<String, failure::Error> {
    let resp = resp.await?;
    debug!("response == {:?}", resp);

    let (_parts, body) = resp.into_parts();
    let complete_body = body::to_bytes(body).await?;

    let s = String::from_utf8(complete_body.to_vec())?;
    trace!("extracted text == {}", s);

    Ok(s)
}

pub async fn create_string_future_from_hyper_request(
    request: hyper::Request<hyper::Body>,
) -> Result<String, failure::Error> {
    let https = hyper_rustls::HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    Ok(extract_body(client.request(request)).await?)
}

pub async fn create_deserialize_future_from_hyper_request<T>(
    request: hyper::Request<hyper::Body>,
) -> Result<T, failure::Error>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    let text = create_string_future_from_hyper_request(request).await?;
    let t = serde_json::from_str(&text)?;
    debug!("deserialized object == {:?}", t);
    Ok(t)
}

async fn serve_function<O, F, Fut>(
    req: Request<Body>,
    f: F,
    options: Arc<O>,
) -> Result<Response<Body>, hyper::Error>
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut,
    Fut: Future<Output = Result<String, failure::Error>>,
    O: std::fmt::Debug,
{
    trace!(
        "serve_function:: req.uri() == {}, req.method() == {}",
        req.uri().path(),
        req.method()
    );
    if req.uri().path() != "/metrics" {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(hyper::Body::empty())
            .unwrap())
    } else if req.method() != "GET" {
        Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(hyper::Body::empty())
            .unwrap())
    } else {
        // everything is ok, let's call the supplied future
        trace!("serve_function:: options == {:?}", options);

        Ok(match f(req, options).await {
            Ok(response) => Response::new(Body::from(response)),
            Err(err) => {
                warn!("internal server error == {:?}", err);

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from(err.to_string()))
                    .unwrap()
            }
        })
    }
}

async fn run_server<O, F, Fut>(addr: SocketAddr, options: Arc<O>, f: F) -> Result<(), hyper::Error>
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut + Send + Clone + Sync + 'static,
    Fut: Future<Output = Result<String, failure::Error>> + Send + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    info!("Listening on http://{}", addr);

    let f = f.clone();
    let options = options.clone();

    let make_service = make_service_fn(move |_| {
        let f = f.clone();
        let options = options.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                serve_function(req, f.clone(), options.clone())
            }))
        }
    });

    let serve_future = Server::bind(&addr).serve(make_service);

    serve_future.await
}

pub async fn render_prometheus<O, F, Fut>(addr: SocketAddr, options: O, f: F)
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut + Send + Clone + Sync + 'static,
    Fut: Future<Output = Result<String, failure::Error>> + Send + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    let o = Arc::new(options);

    let _ = run_server(addr, o, f).await.map_err(|err| {
        error!("{:?}", err);
        eprintln!("Server failure: {:?}", err)
    });
}
