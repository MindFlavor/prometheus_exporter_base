use crate::prometheus_metric_builder::PrometheusMetricBuilder;
use crate::{MetricType, No, RenderToPrometheus};
use supercow::Supercow;

#[derive(Debug)]
pub struct PrometheusMetric<'a> {
    pub counter_name: &'a str,
    pub counter_type: MetricType,
    pub counter_help: &'a str,
    renderable_objects: Vec<Supercow<'a, dyn RenderToPrometheus>>,
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
            renderable_objects: Vec::new(),
        }
    }

    pub fn build() -> PrometheusMetricBuilder<'a, No, No, No> {
        PrometheusMetricBuilder::new()
    }

    pub fn render_header(&self) -> String {
        format!(
            "# HELP {} {}\n# TYPE {} {}\n",
            self.counter_name, self.counter_help, self.counter_name, self.counter_type
        )
    }

    pub fn with_instance(&mut self, renderable_object: &'a dyn RenderToPrometheus) -> &mut Self {
        self.renderable_objects.push(renderable_object);
        self
    }

    pub fn render(&self) -> String {
        let mut s = self.render_header();

        for renderable_object in &self.renderable_objects {
            s.push_str(&format!("{}", self.counter_name));
            let labels = renderable_object.labels();

            if labels.is_empty() {
                s.push_str(&format!(" {}", renderable_object.value()));
            } else {
                s.push_str("{");
                let mut first = true;
                for (key, val) in labels.iter() {
                    if !first {
                        s.push_str(",");
                    } else {
                        first = false;
                    }

                    s.push_str(&format!("{}=\"{}\"", key, val));
                }

                s.push_str(&format!("}} {}", renderable_object.value()));
            }
            if let Some(timestamp) = renderable_object.timestamp() {
                s.push_str(" ");
                s.push_str(&timestamp.to_string());
            }
            s.push_str("\n");
        }

        s
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

        for _ in 0..4 {
            let ret = pc
                .create_instance()
                .with_label("food", "chicken")
                .with_label("instance", &*number.to_string())
                .with_value(number)
                .render();

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
            pc.create_instance().with_value(100).render(),
            format!("gigino_total {}\n", 100)
        );
        assert_eq!(
            pc.create_instance()
                .with_value(100)
                .with_timestamp(9223372036854775807)
                .render(),
            format!("gigino_total 100 9223372036854775807\n")
        );
    }
}
