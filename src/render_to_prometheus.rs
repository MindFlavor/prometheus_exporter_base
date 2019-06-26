use crate::MetricType;

pub trait RenderToPrometheus {
    fn render(&self) -> String;
}

pub struct PrometheusCounter<'a> {
    pub counter_name: &'a str,
    pub counter_type: MetricType,
    pub counter_help: &'a str,
}

impl<'a> PrometheusCounter<'a> {
    pub fn new(
        counter_name: &'a str,
        counter_type: MetricType,
        counter_help: &'a str,
    ) -> PrometheusCounter<'a> {
        PrometheusCounter {
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

    /// Returns the valid Prometheus string, given the optional attributes and the value.
    /// The counter name is in `&self`.
    ///
    /// # Arguments
    ///
    /// * `attributes` - A slice of pairs indicating Attribute-Value. It is optional.
    /// * `value` - A Display-able value that will be appended as counter value.
    ///
    /// # Example
    ///
    /// ```
    ///  use prometheus_exporter_base::{MetricType, PrometheusCounter};
    ///  let pc = PrometheusCounter::new("folder_size", MetricType::Counter, "Size of the folder");
    ///  let mut s = pc.render_header();

    ///  let mut attributes = Vec::new();
    ///  attributes.push(("folder", "/var/log/"));
    ///  s.push_str(&pc.render_counter(Some(&attributes), 1024));

    ///  attributes[0].1 = "/tmp";
    ///  s.push_str(&pc.render_counter(Some(&attributes), 5_000_000));
    /// ```
    pub fn render_counter<N>(&self, attributes: Option<&[(&'a str, &'a str)]>, value: N) -> String
    where
        N: std::fmt::Display,
    {
        if let Some(attributes) = attributes {
            if attributes.is_empty() {
                format!("{} {}\n", self.counter_name, value.to_string())
            } else {
                let mut s = format!("{}{{", self.counter_name);

                let mut first = true;
                for (key, val) in attributes.iter() {
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
        let pc = PrometheusCounter::new("pippo_total", MetricType::Counter, "Number of pippos");

        assert_eq!(
            pc.render_header(),
            "# HELP pippo_total Number of pippos\n# TYPE pippo_total counter\n"
        );
    }

    #[test]
    fn test_attributes() {
        let pc = PrometheusCounter::new("pippo_total", MetricType::Counter, "Number of pippos");
        let mut number = 0;

        let mut attributes = Vec::new();
        attributes.push(("food", "chicken"));
        attributes.push(("instance", ""));

        for _ in 0..4 {
            let mut attributes = Vec::new();
            attributes.push(("food", "chicken"));

            let number_string = number.to_string();
            attributes.push(("instance", &number_string));

            let ret = pc.render_counter(Some(&attributes), &*number.to_string());

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
    fn test_no_attributes() {
        let pc = PrometheusCounter::new("gigino_total", MetricType::Counter, "Number of giginos");
        assert_eq!(
            pc.render_counter(None, 100),
            format!("gigino_total {}\n", 100)
        );
    }
}
