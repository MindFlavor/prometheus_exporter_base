//! This crate simplifies the generation of valid Prometheus metrics. You should start by building
//! a [`PrometheusMetric`] and then adding as many [`PrometheusInstance`] as needed.
//!
//! [`PrometheusMetric`] specifies the [`counter name`], [`type`] and [`help`]. Each [`PrometheusInstance`]
//! specifies the [`value`] and the optional [`labels`] and [`timestamp`].
//!
//! This crate aslo gives you a zero boilerplate `hyper` server behind the `hyper_server` feature
//! gate, check [`render_prometheus`] and the `example` folder
//! for this feature.
//!
//! [`counter name`]: prometheus_metric_builder/struct.PrometheusMetricBuilder.html#method.with_name
//! [`type`]: prometheus_metric_builder/struct.PrometheusMetricBuilder.html#method.with_metric_type
//! [`help`]: prometheus_metric_builder/struct.PrometheusMetricBuilder.html#method.with_help
//! [`counter name`]: struct.PrometheusMetricBuilder.html#method.with_name
//! [`value`]: struct.PrometheusInstance.html#method.with_value
//! [`labels`]: struct.PrometheusInstance.html#method.with_label
//! [`timestamp`]: struct.PrometheusInstance.html#method.with_timestamp
//! [`render_prometheus`]: fn.render_prometheus.html
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

extern crate serde_json;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[cfg(feature = "hyper_server")]
use http::StatusCode;
#[cfg(feature = "hyper_server")]
use hyper::{
    body,
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
#[cfg(feature = "hyper_server")]
use serde::de::DeserializeOwned;
#[cfg(feature = "hyper_server")]
use std::future::Future;
#[cfg(feature = "hyper_server")]
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
pub mod prometheus_metric_builder;
#[cfg(feature = "hyper_server")]
use hyper::http::header::CONTENT_TYPE;
#[cfg(feature = "hyper_server")]
use std::error::Error;
#[cfg(feature = "hyper_server")]
mod server_options;
#[cfg(feature = "hyper_server")]
use server_options::*;

pub trait ToAssign {}
#[derive(Debug, Clone, Copy)]
pub struct Yes {}
#[derive(Debug, Clone, Copy)]
pub struct No {}
impl ToAssign for Yes {}
impl ToAssign for No {}

#[inline]
#[cfg(feature = "hyper_server")]
async fn extract_body(
    resp: hyper::client::ResponseFuture,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let resp = resp.await?;
    debug!("response == {:?}", resp);

    let (_parts, body) = resp.into_parts();
    let complete_body = body::to_bytes(body).await?;

    let s = String::from_utf8(complete_body.to_vec())?;
    trace!("extracted text == {}", s);

    Ok(s)
}

#[cfg(feature = "hyper_server")]
pub async fn create_string_future_from_hyper_request(
    request: hyper::Request<hyper::Body>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()
        .https_only()
        .enable_http1()
        .build();
    let client = Client::builder().build::<_, hyper::Body>(https);

    extract_body(client.request(request)).await
}

#[cfg(feature = "hyper_server")]
pub async fn create_deserialize_future_from_hyper_request<T>(
    request: hyper::Request<hyper::Body>,
) -> Result<T, Box<dyn Error + Send + Sync>>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    let text = create_string_future_from_hyper_request(request).await?;
    let t = serde_json::from_str(&text)?;
    debug!("deserialized object == {:?}", t);
    Ok(t)
}

#[cfg(feature = "hyper_server")]
async fn serve_function<O, F, Fut>(
    server_options: Arc<ServerOptions>,
    req: Request<Body>,
    f: F,
    options: Arc<O>,
) -> Result<Response<Body>, hyper::Error>
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut,
    Fut: Future<Output = Result<String, Box<dyn Error + Send + Sync>>>,
    O: std::fmt::Debug,
{
    trace!(
        "serve_function:: req.uri() == {}, req.method() == {}",
        req.uri().path(),
        req.method()
    );

    trace!(
        "received headers ==> \n{}",
        req.headers()
            .iter()
            .map(|(header_name, header_value)| {
                format!("{} => {}", header_name, header_value.to_str().unwrap())
            })
            .collect::<Vec<_>>()
            .join("\n")
    );

    // check auth if necessary
    let is_authorized = match &server_options.authorization {
        Authorization::Basic(password) => req
            .headers()
            .iter()
            .find(|(header_name, _)| header_name.as_str() == "authorization")
            .map_or_else(
                || Ok::<_, Box<dyn Error + Send + Sync>>(false),
                |(_header_name, header_value)| {
                    let header_value_as_str = header_value.to_str()?;
                    let tokens: Vec<_> = header_value_as_str.split(' ').collect();
                    if tokens.len() != 2 {
                        return Ok(false);
                    }
                    if tokens[0] != "Basic" {
                        return Ok(false);
                    }
                    trace!("Authorization tokens == {:?}", tokens);
                    let base64_decoded = base64::decode(tokens[1])?;
                    let password_from_header = std::str::from_utf8(&base64_decoded)?;
                    Ok(format!(":{}", password) == password_from_header)
                },
            )
            .unwrap_or(false),
        Authorization::None => true,
    };

    if !is_authorized {
        Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(hyper::Body::empty())
            .unwrap())
    } else if req.uri().path() != "/metrics" {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(hyper::Body::empty())
            .unwrap())
    } else if req.method() != "GET" && req.method() != "POST" {
        Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(hyper::Body::empty())
            .unwrap())
    } else {
        // everything is ok, let's call the supplied future
        trace!("serve_function:: options == {:?}", options);

        Ok(match f(req, options).await {
            Ok(response) => Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/plain; version=0.0.4")
                .body(Body::from(response))
                .unwrap(),
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

#[cfg(feature = "hyper_server")]
async fn run_server<O, F, Fut>(
    server_options: ServerOptions,
    options: Arc<O>,
    f: F,
) -> Result<(), hyper::Error>
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut + Send + Clone + Sync + 'static,
    Fut: Future<Output = Result<String, Box<dyn Error + Send + Sync>>> + Send + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    info!("Listening on http://{}/metrics", server_options.addr);

    let f = f.clone();
    let options = options.clone();
    let addr = server_options.addr;
    let server_options = Arc::new(server_options);

    let make_service = make_service_fn(move |_| {
        let f = f.clone();
        let options = options.clone();
        let server_options = server_options.clone();

        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                serve_function(server_options.clone(), req, f.clone(), options.clone())
            }))
        }
    });

    let serve_future = Server::bind(&addr).serve(make_service);

    serve_future.await
}

#[cfg(feature = "hyper_server")]
pub async fn render_prometheus<O, F, Fut>(server_options: ServerOptions, options: O, f: F)
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut + Send + Clone + Sync + 'static,
    Fut: Future<Output = Result<String, Box<dyn Error + Send + Sync>>> + Send + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    let o = Arc::new(options);

    let _ = run_server(server_options, o, f).await.map_err(|err| {
        error!("{:?}", err);
        eprintln!("Server failure: {:?}", err)
    });
}
