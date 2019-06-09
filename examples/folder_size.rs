use clap::{crate_authors, crate_name, crate_version, Arg};
use futures::future::{done, ok, Future};
use hyper::{Body, Response};
use log::{info, trace};
use prometheus_exporter_base::{render_prometheus, PrometheusCounter};
use std::env;
use std::fs::read_dir;

#[derive(Debug, Clone)]
struct MyOptions {}

fn calculate_file_size(path: &str) -> Result<u64, std::io::Error> {
    let mut total_size: u64 = 0;
    for entry in read_dir(path)? {
        let p = entry?.path();
        if p.is_file() {
            total_size += p.metadata()?.len();
        }
    }

    Ok(total_size)
}

fn main() {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::with_name("port")
                .short("p")
                .help("exporter port")
                .default_value("32148")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("verbose logging")
                .takes_value(false),
        )
        .get_matches();

    if matches.is_present("verbose") {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=trace,{}=trace", crate_name!()),
        );
    } else {
        env::set_var(
            "RUST_LOG",
            format!("folder_size=info,{}=info", crate_name!()),
        );
    }
    env_logger::init();

    info!("using matches: {:?}", matches);

    let bind = matches.value_of("port").unwrap();
    let bind = u16::from_str_radix(&bind, 10).expect("port must be a valid number");
    let addr = ([0, 0, 0, 0], bind).into();

    info!("starting exporter on {}", addr);

    render_prometheus(&addr, MyOptions {}, |request, options| {
        Box::new({
            trace!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request,
                options
            );

            // let's calculate the size of /var/log files and /tmp files as an example
            // notice we are using combinators, as soon as async lands this code will be
            // much easier to follow
            let future_log = done(calculate_file_size("/var/log")).from_err();
            let future_tmp = done(calculate_file_size("/tmp")).from_err();
            let joined_future = future_tmp.join(future_log);

            joined_future.and_then(|(total_size_log, total_size_tmp)| {
                let pc = PrometheusCounter::new("folder_size", "counter", "Size of the folder");
                let mut s = pc.render_header();

                let mut attributes = Vec::new();
                attributes.push(("folder", "/var/log/"));
                s.push_str(&pc.render_counter(Some(&attributes), total_size_log));

                attributes[0].1 = "/tmp";
                s.push_str(&pc.render_counter(Some(&attributes), total_size_tmp));

                ok(Response::new(Body::from(s)))
            })
        })
    });
}
