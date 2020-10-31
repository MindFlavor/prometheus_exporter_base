use crate::{RenderToPrometheus, ToAssign, Yes};
use num::Num;
use std::convert::Into;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct MissingValue {}
impl ToAssign for MissingValue {}

#[derive(Debug, Clone)]
pub struct PrometheusInstance<'a, N, ValueSet>
where
    N: Num + std::fmt::Display + std::fmt::Debug,
{
    labels: Vec<(&'a str, &'a str)>,
    value: Option<N>,
    timestamp: Option<i64>,
    value_set: PhantomData<ValueSet>,
}

impl<'a, N> PrometheusInstance<'a, N, MissingValue>
where
    N: Num + std::fmt::Display + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            labels: Vec::new(),
            value: None,
            timestamp: None,
            value_set: PhantomData {},
        }
    }
}

impl<'a, N, ValueSet> PrometheusInstance<'a, N, ValueSet>
where
    N: Num + std::fmt::Display + std::fmt::Debug,
{
    pub fn with_label<L, V>(self, l: L, v: V) -> Self
    where
        L: Into<&'a str>,
        V: Into<&'a str>,
    {
        let mut labels = self.labels;
        labels.push((l.into(), v.into()));

        PrometheusInstance {
            labels,
            value: self.value,
            timestamp: self.timestamp,
            value_set: PhantomData {},
        }
    }

    pub fn with_timestamp(self, timestamp: i64) -> Self {
        PrometheusInstance {
            labels: self.labels,
            value: self.value,
            timestamp: Some(timestamp),
            value_set: PhantomData {},
        }
    }

    pub fn with_value(self, value: N) -> PrometheusInstance<'a, N, Yes> {
        PrometheusInstance {
            labels: self.labels,
            value: Some(value),
            timestamp: self.timestamp,
            value_set: PhantomData {},
        }
    }
}

impl<'a, N> RenderToPrometheus for PrometheusInstance<'a, N, Yes>
where
    N: Num + std::fmt::Display + std::fmt::Debug,
{
    fn render(&self) -> String {
        let mut s = String::new();

        if self.labels.is_empty() {
            s.push_str(&format!(" {}", self.value.as_ref().unwrap().to_string()));
        } else {
            s.push_str("{");
            let mut first = true;
            for (key, val) in self.labels.iter() {
                if !first {
                    s.push_str(",");
                } else {
                    first = false;
                }

                s.push_str(&format!("{}=\"{}\"", key, val));
            }

            s.push_str(&format!("}} {}", self.value.as_ref().unwrap().to_string()));
        }
        if let Some(timestamp) = self.timestamp {
            s.push_str(" ");
            s.push_str(&timestamp.to_string());
        }

        s
    }
}
