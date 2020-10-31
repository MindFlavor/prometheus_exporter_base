/// This trait should be implemented by
/// any instance able to render itself according
/// to Prometheus specifications.
pub trait RenderToPrometheus: std::fmt::Debug {
    /// Render must return the instance formatted
    /// string without the metric info.
    fn render(&self) -> String;
}
