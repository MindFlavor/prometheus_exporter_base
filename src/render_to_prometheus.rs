pub trait RenderToPrometheus: std::fmt::Debug {
    fn render(&self) -> String;
}
