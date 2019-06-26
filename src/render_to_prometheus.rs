use crate::MetricType;
use num::Num;

pub trait RenderToPrometheus {
    fn render(&self) -> String;
}

pub struct PrometheusMetric<'a> {
    pub counter_name: &'a str,
    pub counter_type: MetricType,
    pub counter_help: &'a str,
}

impl<'a> PrometheusMetric<'a> {
    pub fn new(
        counter_name: &'a str,
        counter_type: MetricType,
        counter_help: &'a str,
    ) -> PrometheusMetric<'a> {
        PrometheusMetric {
            counter_name,
            counter_type,
            counter_help,
        }
    }

    pub fn render_header(&self) -> String {
        format!(
            "# HELP {} {}\n# TYPE {} {}\n",
            self.counter_name, self.counter_help, self.counter_name, self.counter_type
        )
    }

    /// Returns the valid Prometheus string, given the optional labels and the value.
    /// The counter name is in `&self`.
    ///
    /// # Arguments
    ///
    /// * `labels` - A slice of pairs indicating Label-Value. It is optional.
    /// * `value` - A number that will be appended as counter value. Remember, all
    /// values in Prometheus are float but you can pass any num::Num values here.
    ///
    /// # Example
    ///
    /// ```
    ///  use prometheus_exporter_base::{MetricType, PrometheusMetric};
    ///  let pc = PrometheusMetric::new("folder_size", MetricType::Counter, "Size of the folder");
    ///  let mut s = pc.render_header();

    ///  let mut labels = Vec::new();
    ///  labels.push(("folder", "/var/log/"));
    ///  s.push_str(&pc.render_sample(Some(&labels), 1024));

    ///  labels[0].1 = "/tmp";
    ///  s.push_str(&pc.render_sample(Some(&labels), 5_000_000));
    /// ```
    pub fn render_sample<N>(&self, labels: Option<&[(&'a str, &'a str)]>, value: N) -> String
    where
        N: Num + std::fmt::Display,
    {
        if let Some(labels) = labels {
            if labels.is_empty() {
                format!("{} {}\n", self.counter_name, value.to_string())
            } else {
                let mut s = format!("{}{{", self.counter_name);

                let mut first = true;
                for (key, val) in labels.iter() {
                    if !first {
                        s.push_str(",");
                    } else {
                        first = false;
                    }

                    s.push_str(&format!("{}=\"{}\"", key, val));
                }

                s.push_str(&format!("}} {}\n", value.to_string()));
                s
            }
        } else {
            format!("{} {}\n", self.counter_name, value.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MetricType;

    #[test]
    fn test_header() {
        let pc = PrometheusMetric::new("pippo_total", MetricType::Counter, "Number of pippos");

        assert_eq!(
            pc.render_header(),
            "# HELP pippo_total Number of pippos\n# TYPE pippo_total counter\n"
        );
    }

    #[test]
    fn test_labels() {
        let pc = PrometheusMetric::new("pippo_total", MetricType::Counter, "Number of pippos");
        let mut number = 0;

        let mut labels = Vec::new();
        labels.push(("food", "chicken"));
        labels.push(("instance", ""));

        for _ in 0..4 {
            let mut labels = Vec::new();
            labels.push(("food", "chicken"));

            let number_string = number.to_string();
            labels.push(("instance", &number_string));

            let ret = pc.render_sample(Some(&labels), number);

            assert_eq!(
                ret,
                format!(
                    "pippo_total{{food=\"chicken\",instance=\"{}\"}} {}\n",
                    number, number
                )
            );
            number += 1;
        }
    }

    #[test]
    fn test_no_labels() {
        let pc = PrometheusMetric::new("gigino_total", MetricType::Counter, "Number of giginos");
        assert_eq!(
            pc.render_sample(None, 100),
            format!("gigino_total {}\n", 100)
        );
    }
}
