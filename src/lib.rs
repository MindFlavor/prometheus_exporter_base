#![feature(async_await)]
extern crate failure;
extern crate serde_json;
use futures::compat::Future01CompatExt;
use futures::future::Future;
use futures::future::{FutureExt, TryFutureExt};
use http::StatusCode;
use hyper::service::service_fn;
use hyper::Client;
use hyper::{Body, Request, Response, Server};
use log::{debug, error, info, trace, warn};
use serde::de::DeserializeOwned;
use std::sync::Arc;
mod render_to_prometheus;
pub use render_to_prometheus::PrometheusMetric;
mod metric_type;
pub use metric_type::MetricType;
use std::net::SocketAddr;

#[inline]
async fn extract_body(req: hyper::client::ResponseFuture) -> Result<String, failure::Error> {
    use futures::compat::Stream01CompatExt;
    use futures::TryStreamExt;

    let resp = req.compat().await?;
    debug!("response == {:?}", resp);

    let (_parts, body) = resp.into_parts();
    let complete_body = body.compat().try_concat().await?;

    let s = String::from_utf8(complete_body.to_vec())?;
    trace!("extracted text == {}", s);

    Ok(s)
}

pub async fn create_string_future_from_hyper_request(
    request: hyper::Request<hyper::Body>,
) -> Result<String, failure::Error> {
    let https = hyper_rustls::HttpsConnector::new(4);
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
        req.uri(),
        req.method()
    );
    if req.uri() != "/metrics" {
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

async fn run_server<O, F, Fut>(addr: SocketAddr, f: F, options: Arc<O>) -> Result<(), hyper::Error>
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<String, failure::Error>> + Send + Sync + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    info!("Listening on http://{}", addr);

    let serve_future = Server::bind(&addr).serve(move || {
        let f = f.clone();
        let options = options.clone();

        service_fn(move |req| {
            serve_function(req, f.clone(), options.clone())
                .boxed()
                .compat()
        })
    });

    serve_future.compat().await
}

pub fn render_prometheus<O, F, Fut>(addr: SocketAddr, f: F, options: O)
where
    F: FnOnce(Request<Body>, Arc<O>) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<String, failure::Error>> + Send + Sync + 'static,
    O: std::fmt::Debug + Sync + Send + 'static,
{
    let o = Arc::new(options);

    let futures_03_future = run_server(addr, f, o);
    let futures_01_future = futures_03_future
        .map_err(|err| {
            error!("{:?}", err);
            eprintln!("Server failure: {:?}", err)
        })
        .boxed()
        .compat();

    // Finally, we can run the future to completion using the `run` function
    // provided by Hyper.
    hyper::rt::run(futures_01_future);
}
