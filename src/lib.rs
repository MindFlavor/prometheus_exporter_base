extern crate failure;
extern crate serde_json;
use futures::future::{done, ok, Either, Future};
use http::StatusCode;
use hyper::service::service_fn;
use hyper::{Body, Request, Response, Server};
use log::{error, trace};
use std::fmt::Debug;
use std::sync::Arc;
mod render_to_prometheus;
pub use render_to_prometheus::PrometheusCounter;

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
        )
            -> Box<Future<Item = Response<Body>, Error = failure::Error> + Send + 'static>
        + Send
        + Clone
        + 'static,
{
    trace!("{:?}", req);

    done(check_compliance(&req)).then(move |res| match res {
        Ok(_) => Either::A(perform_request_box(req, &options).then(|res| match res {
            Ok(body) => ok(body),
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
        )
            -> Box<Future<Item = Response<Body>, Error = failure::Error> + Send + 'static>
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
