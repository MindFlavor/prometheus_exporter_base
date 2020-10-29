use prometheus_exporter_base::{
    render_prometheus, MetricType, PrometheusInstance, PrometheusMetric,
};
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

    render_prometheus(addr, MyOptions::default(), |request, options| async move {
        println!(
            "in our render_prometheus(request == {:?}, options == {:?})",
            request, options
        );

        let total_size_log = calculate_file_size("/var/log").unwrap();

        let pc = PrometheusMetric::build()
            .with_name("folder_size")
            .with_metric_type(MetricType::Counter)
            .with_help("Size of the folder")
            .build();
        let mut s = pc.render_header();

        let mut instance = pc.create_instance();
        instance
            .with_label("folder", "/var/log")
            .with_value(total_size_log);

        let mut attributes = Vec::new();
        attributes.push(("folder", "/var/log/"));
        s.push_str(&pc.render_sample(Some(&attributes), total_size_log, None));

        Ok(s)
    })
    .await;
}
