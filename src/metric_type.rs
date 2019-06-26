macro_rules! string_enum {
    ($name:ident, $error:ident, $($lit:ident),*) => {

        #[derive(Debug, Clone)]
        pub struct $error {
            passed_name: String
        }

        impl $error {
            pub fn passed_name(&self) -> &str {
                &self.passed_name
            }

            pub(crate) fn new(passed_name: &str) -> $error {
                $error { passed_name: passed_name.to_owned() }
            }
        }

        impl std::fmt::Display for $error {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "enum $name does not have the {} variant", self.passed_name
               )
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub enum $name {
            $($lit,)*
        }

        impl std::convert::TryFrom<&str> for $name {
            type Error = $error;

            fn try_from(txt: &str) -> Result<Self, Self::Error> {
                match txt {
                    $(stringify!($lit) => Ok($name::$lit),)*
                    _ => Err($error::new(txt))
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "{}",
                    match self {
                        $($name::$lit => stringify!($lit),)*
                    }
                )
            }
        }
    };
}

string_enum!(
    MetricType,
    UnknownMetricType,
    Counter,
    Gauge,
    Histogram,
    Summary
);
