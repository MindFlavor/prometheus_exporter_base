# Prometheus exporter base

### Base library for Rust Prometheus exporters

master | dev | 
-- | -- |
[![Build Status](https://travis-ci.org/MindFlavor/prometheus_exporter_base.svg?branch=master)](https://travis-ci.org/MindFlavor/prometheus_exporter_base) | [![Build Status](https://travis-ci.org/MindFlavor/prometheus_exporter_base.svg?branch=dev)](https://travis-ci.org/MindFlavor/prometheus_exporter_base)

[![legal](https://img.shields.io/github/license/mindflavor/prometheus_exporter_base.svg)](LICENSE)

[![Crate](https://img.shields.io/crates/v/prometheus_exporter_base.svg)](https://crates.io/crates/prometheus_exporter_base) [![cratedown](https://img.shields.io/crates/d/prometheus_exporter_base.svg)](https://crates.io/crates/prometheus_exporter_base) [![cratelastdown](https://img.shields.io/crates/dv/prometheus_exporter_base.svg)](https://crates.io/crates/prometheus_exporter_base)

[![release](https://img.shields.io/github/release/MindFlavor/prometheus_exporter_base.svg)](https://github.com/MindFlavor/prometheus_exporter_base/tree/0.30.0)
[![tag](https://img.shields.io/github/tag/mindflavor/prometheus_exporter_base.svg)](https://github.com/MindFlavor/prometheus_exporter_base/tree/0.30.0)
[![commitssince](https://img.shields.io/github/commits-since/mindflavor/prometheus_exporter_base/0.30.0.svg)](https://img.shields.io/github/commits-since/mindflavor/prometheus_exporter_base/0.30.0.svg)

## Goal

This crate is meant to make writing a proper Prometheus exporter with a minimal effort. It handles most mundane tasks, such as setting up an Hyper server and doing some basic checks (such as rejecting anything but `GET` and responding only to the `/metrics` suffix) so all you have to do is supply a Boxed future that will handle your logic. I use it on these crates: [prometheus_wireguard_exporter](https://github.com/MindFlavor/prometheus_wireguard_exporter) and [prometheus_iota_exporter](https://github.com/MindFlavor/prometheus_iota_exporter) so please refer to these crates if you want to see a real-world example. More simple examples are available in the [examples](https://github.com/MindFlavor/prometheus_exporter_base/tree/master/examples) folder.

## Usage 

### Main

To use the crate all you have to do is call the `render_prometheus` function. This function requests you to pass: 

1. The address/port to listen to. For example `([0, 0, 0, 0], 32221).into()` listens on every interface on port 32221.
2. An arbitrary struct to be passed back to your code (useful for command line arguments). If you don't need it, pass an empty struct.
3. The *code* your exporter is supposed to do. This takes the form of a closure returning a boxed future. The closure itself will receive the http request data along with the aforementioned struct (point 2). The output is expected to be a string.

For example: 

```rust
render_prometheus(addr, MyOptions::default(), |request, options| {
    async {
    	Ok("it works!".to_owned())
    }
});
```

As you can see, the crate does not enforce anything to the output, it's up to you to return a meaningful string. In order to help the prometheus compliance the crate offers a struct, called `PrometheusMetric`.

### PrometheusMetric

The `PrometheusMetric` struct is used by instantiating it and then "rendering" the header and values - optionally specifying labels. This is an example taken from the documentation: 

```rust
use prometheus_exporter_base::{MetricType, PrometheusMetric};
// create a counter type metric
let pc = PrometheusMetric::new("folder_size", MetricType::Counter, "Size of the folder");
// render the metric header to a string
let mut s = pc.render_header();
// add a label
let mut labels = Vec::new();
labels.push(("folder", "/var/log/"));
// render the sample /var/log with its value
s.push_str(&pc.render_sample(Some(&labels), 1024));
// change the label value to /tmp 
labels[0].1 = "/tmp";
// render the sample /tmp with its value
s.push_str(&pc.render_sample(Some(&labels), 5_000_000));
```

This will give something like this: 

![](extra/001.png)

For a more complete example please refer to the [examples](https://github.com/MindFlavor/prometheus_exporter_base/tree/master/examples) folder.

## Testing

Once running, test your exporter with any GET enabled tool (such as a browser) at `http://127.0.0.1:<your_exporter_port>/metric`.

## License 

Please see the [LICENSE](https://github.com/MindFlavor/prometheus_exporter_base/blob/master/LICENSE) file (spoiler alert: it's MIT).
