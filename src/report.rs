use crate::{location::Location, model::WeatherData, units::Units, Result};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReportKind {
    Forecast,
    History,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u8,
    pub location: Location,
    pub units: Units,
    pub kind: ReportKind,
    pub data: WeatherData,
}
impl Report {
    pub fn new(location: Location, units: Units, kind: ReportKind, data: WeatherData) -> Self {
        Self {
            schema_version: 1,
            location,
            units,
            kind,
            data,
        }
    }
    pub fn json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("cannot encode report: {e}"))
    }
}

impl Report {
    pub fn read(path: &std::path::Path) -> Result<Self> {
        use std::io::Read;
        let file = std::fs::File::open(path).map_err(|e| format!("cannot open report: {e}"))?;
        let mut bytes = Vec::new();
        file.take(crate::http::MAX_RESPONSE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("cannot read report: {e}"))?;
        if bytes.len() > crate::http::MAX_RESPONSE_BYTES {
            return Err("input report is too large".into());
        }
        let report: Self =
            serde_json::from_slice(&bytes).map_err(|e| format!("invalid input report: {e}"))?;
        if report.schema_version != 1 {
            return Err("unsupported report schema version".into());
        }
        if let Some(daily) = &report.data.daily {
            daily.rows()?;
        }
        if let Some(hourly) = &report.data.hourly {
            hourly.rows()?;
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_export_carries_location_units_kind_and_schema_version() {
        let data = serde_json::from_str(r#"{"timezone":"UTC"}"#).unwrap();
        let report = Report::new(
            Location::at(crate::location::Coordinates::new(1.0, 2.0).unwrap()),
            Units::Imperial,
            ReportKind::Forecast,
            data,
        );
        let value: serde_json::Value = serde_json::from_str(&report.json().unwrap()).unwrap();
        assert_eq!(value["schema_version"], 1);
        assert_eq!(value["units"], "imperial");
        assert_eq!(value["kind"], "forecast");
        assert_eq!(value["location"]["coordinates"]["latitude"], 1.0);
        assert!(value["data"]["current"].is_null());
    }
}
