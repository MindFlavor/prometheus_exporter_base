use crate::{MetricType, No, PrometheusMetric, ToAssign, Yes};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct PrometheusMetricBuilder<'a, NameSet, MetricTypeSet, HelpSet>
where
    NameSet: ToAssign,
    MetricTypeSet: ToAssign,
    HelpSet: ToAssign,
{
    p_name: PhantomData<NameSet>,
    p_metric_type: PhantomData<MetricTypeSet>,
    p_help: PhantomData<HelpSet>,
    name: Option<&'a str>,
    metric_type: MetricType,
    help: Option<&'a str>,
}

impl<'a> PrometheusMetricBuilder<'a, No, No, No> {
    #[inline]
    pub(crate) fn new() -> PrometheusMetricBuilder<'a, No, No, No> {
        PrometheusMetricBuilder {
            p_name: PhantomData {},
            name: None,
            p_metric_type: PhantomData {},
            metric_type: MetricType::Gauge,
            p_help: PhantomData {},
            help: None,
        }
    }
}

//get mandatory no traits methods
impl<'a, MetricTypeSet, HelpSet> PrometheusMetricBuilder<'a, Yes, MetricTypeSet, HelpSet>
where
    MetricTypeSet: ToAssign,
    HelpSet: ToAssign,
{
    #[inline]
    pub fn name(&self) -> &'a str {
        self.name.unwrap()
    }
}

impl<'a, NameSet, HelpSet> PrometheusMetricBuilder<'a, NameSet, Yes, HelpSet>
where
    NameSet: ToAssign,
    HelpSet: ToAssign,
{
    #[inline]
    pub fn metric_type(&self) -> MetricType {
        self.metric_type
    }
}

impl<'a, NameSet, MetricTypeSet> PrometheusMetricBuilder<'a, NameSet, MetricTypeSet, Yes>
where
    NameSet: ToAssign,
    MetricTypeSet: ToAssign,
{
    #[inline]
    pub fn help(&self) -> &'a str {
        self.help.unwrap()
    }
}

//set mandatory no traits methods
impl<'a, MetricTypeSet, HelpSet> PrometheusMetricBuilder<'a, No, MetricTypeSet, HelpSet>
where
    MetricTypeSet: ToAssign,
    HelpSet: ToAssign,
{
    #[inline]
    pub fn with_name(
        self,
        name: &'a str,
    ) -> PrometheusMetricBuilder<'a, Yes, MetricTypeSet, HelpSet> {
        PrometheusMetricBuilder {
            p_name: PhantomData {},
            p_metric_type: PhantomData {},
            p_help: PhantomData {},
            name: Some(name),
            metric_type: self.metric_type,
            help: self.help,
        }
    }
}

impl<'a, NameSet, HelpSet> PrometheusMetricBuilder<'a, NameSet, No, HelpSet>
where
    NameSet: ToAssign,
    HelpSet: ToAssign,
{
    #[inline]
    pub fn with_metric_type(
        self,
        metric_type: MetricType,
    ) -> PrometheusMetricBuilder<'a, NameSet, Yes, HelpSet> {
        PrometheusMetricBuilder {
            p_name: PhantomData {},
            p_metric_type: PhantomData {},
            p_help: PhantomData {},
            name: self.name,
            metric_type,
            help: self.help,
        }
    }
}

impl<'a, NameSet, MetricTypeSet> PrometheusMetricBuilder<'a, NameSet, MetricTypeSet, No>
where
    NameSet: ToAssign,
    MetricTypeSet: ToAssign,
{
    #[inline]
    pub fn with_help(
        self,
        help: &'a str,
    ) -> PrometheusMetricBuilder<'a, NameSet, MetricTypeSet, Yes> {
        PrometheusMetricBuilder {
            p_name: PhantomData {},
            p_metric_type: PhantomData {},
            p_help: PhantomData {},
            name: self.name,
            metric_type: self.metric_type,
            help: Some(help),
        }
    }
}

// methods callable regardless
impl<'a, NameSet, MetricTypeSet, HelpSet>
    PrometheusMetricBuilder<'a, NameSet, MetricTypeSet, HelpSet>
where
    NameSet: ToAssign,
    MetricTypeSet: ToAssign,
    HelpSet: ToAssign,
{
}

// methods callable only when every mandatory field has been filled
impl<'a> PrometheusMetricBuilder<'a, Yes, Yes, Yes> {
    pub fn build(self) -> PrometheusMetric<'a> {
        PrometheusMetric::new(self.name(), self.metric_type, self.help())
    }
}
