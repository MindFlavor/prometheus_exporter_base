use crate::{RenderToPrometheus, ToAssign, Yes};
use std::convert::Into;
use std::marker::PhantomData;
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

#[derive(Debug, Clone, Copy)]
pub struct MissingValue {}
impl ToAssign for MissingValue {}

#[derive(Debug, Clone)]
pub struct PrometheusInstance<'a, N, ValueSet>
where
    N: std::fmt::Display + std::fmt::Debug,
{
    labels: Vec<(&'a str, &'a str)>,
    value: Option<N>,
    timestamp: Option<u128>,
    value_set: PhantomData<ValueSet>,
}

impl<'a, N> PrometheusInstance<'a, N, MissingValue>
where
    N: std::fmt::Display + std::fmt::Debug,
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

impl<'a, N> Default for PrometheusInstance<'a, N, MissingValue>
where
    N: std::fmt::Display + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, N, ValueSet> PrometheusInstance<'a, N, ValueSet>
where
    N: std::fmt::Display + std::fmt::Debug,
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

    /// Adds the optional timestamp to the instance.
    ///
    /// Example:
    ///
    /// ```
    /// use prometheus_exporter_base::prelude::*;
    ///
    /// PrometheusInstance::new()
    ///     .with_timestamp(123)
    ///     .with_value(123);
    /// ```
    pub fn with_timestamp(self, timestamp: u128) -> Self {
        PrometheusInstance {
            labels: self.labels,
            value: self.value,
            timestamp: Some(timestamp),
            value_set: PhantomData {},
        }
    }

    /// Adds the current timestamp to the instance. The timestamp
    /// is calculated as milliseconds from the current `UNIX_EPOCH` as per
    /// specification.
    /// Example:
    ///
    /// ```
    /// use prometheus_exporter_base::prelude::*;
    ///
    /// PrometheusInstance::new()
    ///     .with_current_timestamp()
    ///     .expect("failed to get the UNIX epoch")
    ///     .with_value(123);
    /// ```
    pub fn with_current_timestamp(self) -> Result<Self, SystemTimeError> {
        Ok(PrometheusInstance {
            labels: self.labels,
            value: self.value,
            timestamp: Some(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()),
            value_set: PhantomData {},
        })
    }

    /// Adds the current value to the instance. The value
    /// will be formatted as float as per
    /// specification.
    ///
    /// Example:
    ///
    /// ```
    /// use prometheus_exporter_base::prelude::*;
    ///
    /// PrometheusInstance::new()
    ///     .with_value(123);
    /// ```
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
    N: std::fmt::Display + std::fmt::Debug,
{
    fn render(&self) -> String {
        let mut s = String::new();

        if self.labels.is_empty() {
            s.push_str(&format!(" {}", self.value.as_ref().unwrap()));
        } else {
            s.push('{');
            let mut first = true;
            for (key, val) in self.labels.iter() {
                if !first {
                    s.push(',');
                } else {
                    first = false;
                }

                s.push_str(&format!("{}=\"{}\"", key, val));
            }

            s.push_str(&format!("}} {}", self.value.as_ref().unwrap()));
        }
        if let Some(timestamp) = self.timestamp {
            s.push(' ');
            s.push_str(&timestamp.to_string());
        }

        s
    }
}
