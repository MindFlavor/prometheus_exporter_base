use clap::{crate_authors, crate_name, crate_version, value_parser, Arg, ArgAction};
use log::{info, trace};
use prometheus_exporter_base::prelude::*;
use std::env;
use std::fs::read_dir;
use std::net::SocketAddr;

#[derive(Debug, Clone, Default)]
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

#[tokio::main]
async fn main() {
    let matches = clap::Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .arg(
            Arg::new("port")
                .short('p')
                .help("exporter port")
                .value_parser(value_parser!(u16))
                .default_value("32148"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .help("verbose logging")
                .action(ArgAction::Count),
        )
        .get_matches();

    if matches.get_count("verbose") > 0 {
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

    let bind: u16 = *matches.get_one("port").unwrap();
    let addr: SocketAddr = ([0, 0, 0, 0], bind).into();

    let server_options = ServerOptions {
        addr,
        authorization: Authorization::None,
    };
    println!("starting exporter with options {:?}", addr);

    render_prometheus(
        server_options,
        MyOptions::default(),
        |request, options| async move {
            trace!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request,
                options
            );

            let mut pc = PrometheusMetric::build()
                .with_name("folder_size")
                .with_metric_type(MetricType::Counter)
                .with_help("Size of the folder")
                .build();

            for folder in &vec!["/var/log", "/tmp"] {
                pc.render_and_append_instance(
                    &PrometheusInstance::new()
                        .with_label("folder", folder.as_ref())
                        .with_value(
                            calculate_file_size(folder).expect("cannot calculate folder size"),
                        )
                        .with_current_timestamp()
                        .expect("error getting the current UNIX epoch"),
                );
            }

            Ok(pc.render())
        },
    )
    .await;
}
