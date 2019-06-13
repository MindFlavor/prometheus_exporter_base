extern crate failure;
extern crate serde_json;
use futures::future::{done, ok, Either, Future};
use futures::stream::Stream;
use http::StatusCode;
use hyper::service::service_fn;
use hyper::Client;
use hyper::{Body, Request, Response, Server};
use log::{debug, error, trace};
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::sync::Arc;
mod render_to_prometheus;
pub use render_to_prometheus::PrometheusCounter;

#[inline]
fn extract_body(
    req: hyper::client::ResponseFuture,
) -> impl Future<Item = String, Error = failure::Error> + Send {
    req.from_err().and_then(|resp| {
        debug!("response == {:?}", resp);
        let (_parts, body) = resp.into_parts();
        body.concat2()
            .from_err()
            .and_then(|complete_body| done(String::from_utf8(complete_body.to_vec())).from_err())
    })
}

pub fn create_string_future_from_hyper_request(
    request: hyper::Request<hyper::Body>,
) -> impl Future<Item = String, Error = failure::Error> {
    let https = hyper_rustls::HttpsConnector::new(4);
    let client = Client::builder().build::<_, hyper::Body>(https);

    extract_body(client.request(request))
        .from_err()
        .and_then(|text: String| {
            debug!("received_text == {:?}", text);
            ok(text)
        })
}

pub fn create_deserialize_future_from_hyper_request<T>(
    request: hyper::Request<hyper::Body>,
) -> impl Future<Item = T, Error = failure::Error>
where
    T: DeserializeOwned + std::fmt::Debug,
{
    create_string_future_from_hyper_request(request)
        .and_then(|text| done(serde_json::from_str(&text)).from_err())
        .and_then(|t: T| {
            debug!("deserialized object == {:?}", t);
            ok(t)
        })
}

fn check_compliance(req: &Request<Body>) -> Result<(), Response<Body>> {
    if req.uri() != "/metrics" {
        trace!("uri not allowed {}", req.uri());
        Err(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(hyper::Body::empty())
            .unwrap())
    } else if req.method() != "GET" {
        trace!("method not allowed {}", req.method());
        Err(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body(hyper::Body::empty())
            .unwrap())
    } else {
        Ok(())
    }
}

fn handle_request<O, P>(
    req: Request<Body>,
    options: Arc<O>,
    perform_request_box: P,
) -> impl Future<Item = Response<Body>, Error = failure::Error>
where
    O: Debug + Clone + Send + Sync + 'static,
    P: FnOnce(
            Request<Body>,
            &Arc<O>,
        ) -> Box<Future<Item = String, Error = failure::Error> + Send + 'static>
        + Send
        + Clone
        + 'static,
{
    trace!("{:?}", req);

    done(check_compliance(&req)).then(move |res| match res {
        Ok(_) => Either::A(perform_request_box(req, &options).then(|res| match res {
            Ok(body) => ok(Response::new(Body::from(body))),
            Err(err) => {
                error!("internal server error: {:?}", err);
                ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(hyper::Body::empty())
                    .unwrap())
            }
        })),
        Err(err) => Either::B(ok(err)),
    })
}

pub fn render_prometheus<O, P>(addr: &::std::net::SocketAddr, options: O, perform_request: P)
where
    O: Debug + Clone + Send + Sync + 'static,
    P: FnOnce(
            Request<Body>,
            &Arc<O>,
        ) -> Box<Future<Item = String, Error = failure::Error> + Send + 'static>
        + Send
        + Clone
        + 'static,
{
    // let's avoid unnecessary copies of our readonly data
    let options = Arc::new(options.clone());

    let new_svc = move || {
        let options = options.clone();
        let perform_request = perform_request.clone();
        service_fn(move |req| handle_request(req, options.clone(), perform_request.clone()))
    };

    let server = Server::bind(&addr)
        .serve(new_svc)
        .map_err(|e| eprintln!("server error: {}", e));
    hyper::rt::run(server);
}
