use crate::{PrometheusMetric, ToAssign, Yes};
use num::Num;
use std::convert::Into;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub struct MissingValue {}
impl ToAssign for MissingValue {}

#[derive(Debug, Clone)]
pub struct PrometheusInstance<'a, N, ValueSet>
where
    N: Num + std::fmt::Display,
{
    metric: &'a PrometheusMetric<'a>,
    labels: Option<Vec<(&'a str, &'a str)>>,
    value: Option<N>,
    timestamp: Option<i64>,
    value_set: PhantomData<ValueSet>,
}

impl<'a, N> PrometheusInstance<'a, N, MissingValue>
where
    N: Num + std::fmt::Display,
{
    pub(crate) fn new(metric: &'a PrometheusMetric) -> Self {
        Self {
            metric,
            labels: None,
            value: None,
            timestamp: None,
            value_set: PhantomData {},
        }
    }
}

impl<'a, N, ValueSet> PrometheusInstance<'a, N, ValueSet>
where
    N: Num + std::fmt::Display,
{
    pub fn with_label<L, V>(&mut self, l: L, v: V) -> &mut Self
    where
        L: Into<&'a str>,
        V: Into<&'a str>,
    {
        if let Some(labels) = &mut self.labels {
            labels.push((l.into(), v.into()));
        } else {
            let mut labels = Vec::new();
            labels.push((l.into(), v.into()));
            self.labels = Some(labels);
        }

        self
    }

    pub fn with_timestamp(&mut self, timestamp: i64) -> &mut Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn value(self, value: N) -> PrometheusInstance<'a, N, Yes> {
        PrometheusInstance {
            metric: self.metric,
            labels: self.labels,
            value: Some(value),
            timestamp: self.timestamp,
            value_set: PhantomData {},
        }
    }
}

impl<'a, N> PrometheusInstance<'a, N, Yes>
where
    N: Num + std::fmt::Display,
{
    pub fn render(&self) -> String {
        // this is safe because of the type
        let value = self.value.as_ref().unwrap();

        let mut s = format!("{}", self.metric.counter_name);
        if let Some(labels) = &self.labels {
            if labels.is_empty() {
                s.push_str(&format!(" {}", value.to_string()));
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

                s.push_str(&format!("}} {}", value.to_string()));
            }
        } else {
            s.push_str(" ");
            s.push_str(&value.to_string());
        }
        if let Some(timestamp) = self.timestamp {
            s.push_str(" ");
            s.push_str(&timestamp.to_string());
        }
        s.push_str("\n");
        s
    }
}
