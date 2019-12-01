use prometheus_exporter_base::{render_prometheus, MetricType, PrometheusMetric};
use std::fs::read_dir;

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
    let addr = ([0, 0, 0, 0], 32221).into();
    println!("starting exporter on {}", addr);

    render_prometheus(addr, MyOptions::default(), |request, options| {
        async move {
            println!(
                "in our render_prometheus(request == {:?}, options == {:?})",
                request, options
            );

            let total_size_log = calculate_file_size("/var/log").unwrap();

            let pc =
                PrometheusMetric::new("folder_size", MetricType::Counter, "Size of the folder");
            let mut s = pc.render_header();

            let mut attributes = Vec::new();
            attributes.push(("folder", "/var/log/"));
            s.push_str(&pc.render_sample(Some(&attributes), total_size_log));

            Ok(s)
        }
    })
    .await;
}
