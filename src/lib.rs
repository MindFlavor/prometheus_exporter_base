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

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

#[cfg(feature = "hyper_server")]
use http::StatusCode;
#[cfg(feature = "hyper_server")]
use hyper::http::header::CONTENT_TYPE;
#[cfg(feature = "hyper_server")]
use hyper::{service::service_fn, Request, Response};
#[cfg(feature = "hyper_server")]
use rustls::{
    pki_types::{CertificateDer, PrivateKeyDer},
    server::VerifierBuilderError,
};
#[cfg(feature = "hyper_server")]
use std::error::Error;
#[cfg(feature = "hyper_server")]
use std::future::Future;
#[cfg(feature = "hyper_server")]
use std::sync::Arc;
#[cfg(feature = "hyper_server")]
use std::{fs, io};
#[cfg(feature = "hyper_server")]
use thiserror::Error;
#[cfg(feature = "hyper_server")]
mod server_options;
#[cfg(feature = "hyper_server")]
use http_body_util::Full;
#[cfg(feature = "hyper_server")]
use hyper::body::Bytes;
#[cfg(feature = "hyper_server")]
use hyper::server::conn::http1;
#[cfg(feature = "hyper_server")]
use hyper_util::rt::TokioIo;
#[cfg(feature = "hyper_server")]
use rustls::{server::WebPkiClientVerifier, RootCertStore, ServerConfig};
#[cfg(feature = "hyper_server")]
use server_options::*;
#[cfg(feature = "hyper_server")]
use tokio::net::TcpListener;
#[cfg(feature = "hyper_server")]
use tokio_rustls::TlsAcceptor;

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

pub trait ToAssign {}
#[derive(Debug, Clone, Copy)]
pub struct Yes {}
#[derive(Debug, Clone, Copy)]
pub struct No {}
impl ToAssign for Yes {}
impl ToAssign for No {}

#[cfg(feature = "hyper_server")]
async fn serve_function<O, F, Fut>(
    server_options: Arc<ServerOptions>,
    req: Request<hyper::body::Incoming>,
    f: F,
    options: Arc<O>,
) -> Result<Response<Full<Bytes>>, hyper::Error>
where
    F: FnOnce(Request<hyper::body::Incoming>, Arc<O>) -> Fut,
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
            .body(Full::default())
            .unwrap())
    } else if req.uri().path() != "/metrics" {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::default())
            .unwrap())
    } else if req.method() != "GET" {
        Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(Full::default())
            .unwrap())
    } else {
        // everything is ok, let's call the supplied future
        trace!("serve_function:: options == {:?}", options);

        Ok(match f(req, options).await {
            Ok(response) => Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/plain; version=0.0.4")
                .body(response.into())
                .unwrap(),
            Err(err) => {
                warn!("internal server error == {:?}", err);

                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(err.to_string().into())
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
) -> Result<(), ServerError>
where
    F: FnOnce(Request<hyper::body::Incoming>, Arc<O>) -> Fut + Send + Clone + Sync + 'static,
    Fut: Future<Output = Result<String, Box<dyn Error + Send + Sync>>> + Send + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    let server_options = Arc::new(server_options);

    let service = {
        let f = f.clone();
        let server_options = server_options.clone();
        service_fn(move |req: Request<hyper::body::Incoming>| {
            let f = f.clone();
            let options = options.clone();
            let server_options = server_options.clone();
            async move { serve_function(server_options, req, f, options).await }
        })
    };

    let tcp_listener = TcpListener::bind(&server_options.addr).await?;
    let tls_acceptor = if let Some(tls_options) = &server_options.tls_options {
        let server_config = if let Some(client_ca) = &tls_options.client_certificate_ca_file {
            let mut root_store = RootCertStore::empty();
            for root in load_certs(client_ca)? {
                root_store.add(root)?;
            }
            ServerConfig::builder().with_client_cert_verifier(
                WebPkiClientVerifier::builder(Arc::new(root_store)).build()?,
            )
        } else {
            ServerConfig::builder().with_no_client_auth()
        }
        .with_single_cert(
            load_certs(&tls_options.certificate_chain_file)?,
            load_private_key(&tls_options.key_file)?,
        )?;

        info!("Listening on https://{}/metrics", server_options.addr);
        Some(TlsAcceptor::from(Arc::new(server_config)))
    } else {
        info!("Listening on http://{}/metrics", server_options.addr);
        None
    };

    loop {
        let service = service.clone();
        let (tcp_stream, _remote_addr) = tcp_listener.accept().await?;
        let tls_acceptor = tls_acceptor.clone();
        tokio::spawn(async move {
            if let Some(tls_acceptor) = tls_acceptor {
                let tls_stream = match tls_acceptor.accept(tcp_stream).await {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("tls error: {e}");
                        return;
                    }
                };
                if let Err(e) = http1::Builder::new()
                    .serve_connection(TokioIo::new(tls_stream), service)
                    .await
                {
                    eprintln!("failed to serve connection: {e}");
                }
            } else if let Err(e) = http1::Builder::new()
                .serve_connection(TokioIo::new(tcp_stream), service)
                .await
            {
                eprintln!("failed to serve connection: {e}");
            }
        });
    }
}

#[cfg(feature = "hyper_server")]
#[derive(Debug, Error)]
enum ServerError {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("server error: {0}")]
    ServerError(#[from] hyper::Error),
    #[error("client certificate verifier error: {0}")]
    ClientCertVerifierError(#[from] VerifierBuilderError),
    #[error("rustls error: {0}")]
    RustlsError(#[from] rustls::Error),
}

#[cfg(feature = "hyper_server")]
pub async fn render_prometheus<O, F, Fut>(server_options: ServerOptions, options: O, f: F)
where
    F: FnOnce(Request<hyper::body::Incoming>, Arc<O>) -> Fut + Send + Clone + Sync + 'static,
    Fut: Future<Output = Result<String, Box<dyn Error + Send + Sync>>> + Send + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    let o = Arc::new(options);

    let _ = run_server(server_options, o, f).await.map_err(|err| {
        error!("{:?}", err);
        eprintln!("Server failure: {:?}", err)
    });
}

#[cfg(feature = "hyper_server")]
// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    // Open certificate file.
    let certfile = fs::File::open(filename).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("failed to open {}: {}", filename, e),
        )
    })?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    rustls_pemfile::certs(&mut reader).collect()
}

#[cfg(feature = "hyper_server")]
// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    // Open keyfile.
    let keyfile = fs::File::open(filename).map_err(|e| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("failed to open {}: {}", filename, e),
        )
    })?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}
