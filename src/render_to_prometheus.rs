use crate::prometheus_metric_builder::PrometheusMetricBuilder;
use crate::{MetricType, MissingValue, No, PrometheusInstance};
use num::Num;

pub trait RenderToPrometheus {
    fn render(&self) -> String;
}

#[derive(Debug, Clone)]
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

    pub fn create_instance<N>(&'a self) -> PrometheusInstance<'a, N, MissingValue>
    where
        N: Num + std::fmt::Display,
    {
        self.into()
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
}

impl<'a, N> Into<PrometheusInstance<'a, N, MissingValue>> for &'a PrometheusMetric<'a>
where
    N: Num + std::fmt::Display,
{
    fn into(self) -> PrometheusInstance<'a, N, MissingValue> {
        PrometheusInstance::new(self)
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
