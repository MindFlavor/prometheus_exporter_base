pub trait RenderToPrometheus: std::fmt::Debug {
    fn labels(&self) -> &[(&str, &str)];
    fn value(&self) -> String;
    fn timestamp(&self) -> Option<i64>;
}
