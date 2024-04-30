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
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err("unsupported report schema version".into());
        }
        if self.location.name.trim().is_empty() || self.location.name.chars().count() > 200 {
            return Err("invalid report location name".into());
        }
        if self
            .location
            .country
            .as_deref()
            .is_some_and(|c| c.len() != 2 || !c.bytes().all(|b| b.is_ascii_alphabetic()))
        {
            return Err("invalid report country code".into());
        }
        self.data.validate()?;
        let rows = self
            .data
            .daily
            .as_ref()
            .ok_or("report is missing daily data")?
            .rows()?;
        match self.kind {
            ReportKind::Forecast => {
                if rows.len() > 16 {
                    return Err("forecast exceeds sixteen days".into());
                }
                self.data.forecast_coverage(rows.len() as u8)?;
            }
            ReportKind::History if self.data.current.is_some() || self.data.hourly.is_some() => {
                return Err("history report cannot contain current or hourly forecasts".into())
            }
            ReportKind::History => {}
        }
        Ok(())
    }
    pub fn validate_for(&self, args: &crate::cli::Options) -> Result<()> {
        self.validate()?;
        if self.units != args.units {
            return Err("cached units do not match the request".into());
        }
        if let Some(coordinates) = args.coordinates()? {
            if coordinates != self.location.coordinates {
                return Err("cached coordinates do not match the request".into());
            }
        }
        if let Some(country) = &args.country_code {
            if !self
                .location
                .country
                .as_deref()
                .is_some_and(|c| c.eq_ignore_ascii_case(country))
            {
                return Err("cached country does not match the request".into());
            }
        }
        if let Some(range) = args.history_range()? {
            if self.kind != ReportKind::History {
                return Err("cached report kind does not match a history request".into());
            }
            self.data.history_coverage(range)?;
        } else {
            if self.kind != ReportKind::Forecast {
                return Err("cached report kind does not match a forecast request".into());
            }
            self.data.forecast_coverage(args.days)?;
        }
        Ok(())
    }
    pub fn json(&self) -> Result<String> {
        self.validate()?;
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
        report.validate()?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_export_carries_location_units_kind_and_schema_version() {
        let mut data =
            serde_json::from_str::<Report>(include_str!("../tests/fixtures/forecast.json"))
                .unwrap()
                .data;
        data.current = None;
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
