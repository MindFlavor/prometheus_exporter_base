use futures::future::{done, ok, Future};
use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusCounter};
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
    let addr = ([0, 0, 0, 0], 32221).into();
    println!("starting exporter on {}", addr);

    render_prometheus(&addr, MyOptions {}, |request, options| {
        Box::new({
            println!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request, options
            );

            let future_log = done(calculate_file_size("/var/log")).from_err();
            future_log.and_then(|total_size_log| {
                let pc = PrometheusCounter::new(
                    "folder_size",
                    MetricType::Counter,
                    "Size of the folder",
                );
                let mut s = pc.render_header();

                let mut attributes = Vec::new();
                attributes.push(("folder", "/var/log/"));
                s.push_str(&pc.render_counter(Some(&attributes), total_size_log));

                ok(s)
            })
        })
    });
}
