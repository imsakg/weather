use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Units {
    #[default]
    Metric,
    Imperial,
}
impl Units {
    pub fn temperature_label(self) -> &'static str { match self { Self::Metric => "°C", Self::Imperial => "°F" } }
    pub fn temperature_parameter(self) -> &'static str { match self { Self::Metric => "celsius", Self::Imperial => "fahrenheit" } }
    pub fn wind_parameter(self) -> &'static str { match self { Self::Metric => "kmh", Self::Imperial => "mph" } }
    pub fn wind_label(self) -> &'static str { match self { Self::Metric => "km/h", Self::Imperial => "mph" } }
    pub fn precipitation_parameter(self) -> &'static str { match self { Self::Metric => "mm", Self::Imperial => "inch" } }
    pub fn precipitation_label(self) -> &'static str { match self { Self::Metric => "mm", Self::Imperial => "in" } }
}
impl FromStr for Units {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value { "metric" => Ok(Self::Metric), "imperial" => Ok(Self::Imperial), _ => Err("units must be metric or imperial".into()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_unit_systems_without_silent_fallback() {
        assert_eq!("metric".parse::<Units>().unwrap(), Units::Metric);
        assert_eq!("imperial".parse::<Units>().unwrap(), Units::Imperial);
        assert!("kelvin".parse::<Units>().is_err());
        assert_eq!(Units::Metric.temperature_label(), "°C");
        assert_eq!(Units::Imperial.temperature_label(), "°F");
        assert_eq!(Units::Imperial.wind_parameter(), "mph");
    }
}
