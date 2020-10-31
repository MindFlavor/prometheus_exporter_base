use crate::prometheus_metric_builder::PrometheusMetricBuilder;
use crate::{MetricType, No, RenderToPrometheus};

#[derive(Debug)]
pub struct PrometheusMetric<'a> {
    pub(crate) counter_name: &'a str,
    pub(crate) counter_type: MetricType,
    pub(crate) counter_help: &'a str,
    pub(crate) rendered_instances: Vec<String>,
}

impl<'a> PrometheusMetric<'a> {
    #[deprecated(since = "1.0.0", note = "Please use the build function instead")]
    pub fn new(
        counter_name: &'a str,
        counter_type: MetricType,
        counter_help: &'a str,
    ) -> PrometheusMetric<'a> {
        PrometheusMetric {
            counter_name,
            counter_type,
            counter_help,
            rendered_instances: Vec::new(),
        }
    }

    /// Call this function to construct a
    /// [`PrometheusMetric`].
    ///
    /// Example:
    ///
    /// ```
    /// use prometheus_exporter_base::prelude::*;
    ///
    /// let prometheus_metric = PrometheusMetric::build()
    ///     .with_name("folder_size")
    ///     .with_metric_type(MetricType::Counter)
    ///     .with_help("Size of the folder")
    ///     .build();
    /// ```
    pub fn build() -> PrometheusMetricBuilder<'a, No, No, No> {
        PrometheusMetricBuilder::new()
    }

    fn render_header(&self) -> String {
        format!(
            "# HELP {} {}\n# TYPE {} {}\n",
            self.counter_name, self.counter_help, self.counter_name, self.counter_type
        )
    }

    /// Call this function to add a [`PrometheusInstance`] rendered
    /// String to the instances list. You can call this function as many
    /// times as needed. It's up to you to give meaningful
    /// labels however.
    ///
    /// **Note**: the instance will be rendered immediately so you can
    /// reuse the [`PrometheusInstance`] if needed.
    ///
    /// [`PrometheusInstance`]: struct.PrometheusInstance.html
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// use prometheus_exporter_base::prelude::*;
    ///
    /// let mut pc = PrometheusMetric::build()
    ///        .with_name("folder_size")
    ///        .with_metric_type(MetricType::Counter)
    ///        .with_help("Size of the folder")
    ///        .build();
    ///
    /// for folder in &vec!["/var/log", "/tmp"] {
    ///     pc.render_and_append_instance(
    ///         &PrometheusInstance::new()
    ///             .with_label("folder", folder.as_ref())
    ///             .with_value(500) // this is just an example!
    ///             .with_current_timestamp()
    ///             .expect("error getting the current UNIX epoch"),
    ///     );
    /// }
    ///
    /// let final_string = pc.render();
    ///
    /// ```
    pub fn render_and_append_instance(
        &mut self,
        rendereable_instance: &dyn RenderToPrometheus,
    ) -> &mut Self {
        self.rendered_instances.push(rendereable_instance.render());
        self
    }

    pub fn render(&self) -> String {
        let mut s = self.render_header();

        for rendered_instance in &self.rendered_instances {
            s.push_str(&format!("{}{}", self.counter_name, rendered_instance));
            s.push_str("\n");
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MetricType, PrometheusInstance};

    #[test]
    fn test_header() {
        let pc = PrometheusMetric::build()
            .with_name("pippo_total")
            .with_metric_type(MetricType::Counter)
            .with_help("Number of pippos")
            .build();

        assert_eq!(
            pc.render_header(),
            "# HELP pippo_total Number of pippos\n# TYPE pippo_total counter\n"
        );
    }

    #[test]
    fn test_labels() {
        let mut pc = PrometheusMetric::build()
            .with_name("pippo_total")
            .with_metric_type(MetricType::Counter)
            .with_help("Number of pippos")
            .build();

        for number in 0..4 {
            pc.render_and_append_instance(
                &PrometheusInstance::new()
                    .with_label("food", "chicken")
                    .with_label("instance", &*number.to_string())
                    .with_value(number),
            );
        }

        assert_eq!(
            pc.render(),
            "# HELP pippo_total Number of pippos\n\
        # TYPE pippo_total counter\n\
        pippo_total{food=\"chicken\",instance=\"0\"} 0\n\
        pippo_total{food=\"chicken\",instance=\"1\"} 1\n\
        pippo_total{food=\"chicken\",instance=\"2\"} 2\n\
        pippo_total{food=\"chicken\",instance=\"3\"} 3\n"
        );
    }

    #[test]
    fn test_no_labels() {
        let final_string = PrometheusMetric::build()
            .with_name("gigino_total")
            .with_metric_type(MetricType::Counter)
            .with_help("Number of giginos")
            .build()
            .render_and_append_instance(&PrometheusInstance::new().with_value(100))
            .render();

        assert_eq!(
            final_string,
            "# HELP gigino_total Number of giginos\n\
        # TYPE gigino_total counter\n\
        gigino_total 100\n"
        );

        let final_string = PrometheusMetric::build()
            .with_name("gigino_total")
            .with_metric_type(MetricType::Counter)
            .with_help("Number of giginos")
            .build()
            .render_and_append_instance(
                &PrometheusInstance::new()
                    .with_value(100)
                    .with_timestamp(9223372036854775807),
            )
            .render();

        assert_eq!(
            final_string,
            "# HELP gigino_total Number of giginos\n\
        # TYPE gigino_total counter\n\
        gigino_total 100 9223372036854775807\n"
        );
    }
}
