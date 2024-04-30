use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub timezone: String,
    pub current: Option<Current>,
    pub daily: Option<Daily>,
    pub hourly: Option<Hourly>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Current {
    pub time: String,
    pub temperature_2m: Option<f64>,
    pub relative_humidity_2m: Option<f64>,
    pub apparent_temperature: Option<f64>,
    pub weather_code: Option<u16>,
    pub wind_speed_10m: Option<f64>,
}
pub fn condition(code: Option<u16>) -> &'static str {
    match code {
        Some(0) => "Clear sky",
        Some(1) => "Mainly clear",
        Some(2) => "Partly cloudy",
        Some(3) => "Overcast",
        Some(45 | 48) => "Fog",
        Some(51 | 53 | 55) => "Drizzle",
        Some(56 | 57) => "Freezing drizzle",
        Some(61 | 63 | 65) => "Rain",
        Some(66 | 67) => "Freezing rain",
        Some(71 | 73 | 75 | 77) => "Snow",
        Some(80..=82) => "Rain showers",
        Some(85 | 86) => "Snow showers",
        Some(95) => "Thunderstorm",
        Some(96 | 99) => "Thunderstorm with hail",
        _ => "Unknown",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Daily {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<Option<f64>>,
    pub temperature_2m_min: Vec<Option<f64>>,
    pub precipitation_sum: Vec<Option<f64>>,
    pub weather_code: Vec<Option<u16>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Day {
    pub date: String,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub rain: Option<f64>,
    pub code: Option<u16>,
}
impl Daily {
    pub fn rows(&self) -> crate::Result<Vec<Day>> {
        let len = self.time.len();
        if [
            self.temperature_2m_max.len(),
            self.temperature_2m_min.len(),
            self.precipitation_sum.len(),
            self.weather_code.len(),
        ]
        .iter()
        .any(|&n| n != len)
        {
            return Err("provider returned mismatched daily arrays".into());
        }
        if len == 0 || len > 366 {
            return Err("daily data must contain 1..366 days".into());
        }
        let mut previous: Option<chrono::NaiveDate> = None;
        for i in 0..len {
            let date = calendar_date(&self.time[i])?;
            if previous.is_some_and(|p| (date - p).num_days() != 1) {
                return Err("daily dates must be consecutive and ascending".into());
            }
            previous = Some(date);
            measurement(self.temperature_2m_max[i], None, None, "daily high")?;
            measurement(self.temperature_2m_min[i], None, None, "daily low")?;
            measurement(self.precipitation_sum[i], Some(0.0), None, "precipitation")?;
            if self.temperature_2m_max[i]
                .zip(self.temperature_2m_min[i])
                .is_some_and(|(high, low)| high < low)
            {
                return Err("daily high cannot be below daily low".into());
            }
        }
        Ok((0..len)
            .map(|i| Day {
                date: self.time[i].clone(),
                high: self.temperature_2m_max[i],
                low: self.temperature_2m_min[i],
                rain: self.precipitation_sum[i],
                code: self.weather_code[i],
            })
            .collect())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hourly {
    pub time: Vec<String>,
    pub temperature_2m: Vec<Option<f64>>,
    pub precipitation_probability: Vec<Option<f64>>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hour {
    pub time: String,
    pub temperature: Option<f64>,
    pub rain_probability: Option<f64>,
}
impl Hourly {
    pub fn rows(&self) -> crate::Result<Vec<Hour>> {
        if self.time.len() != self.temperature_2m.len()
            || self.time.len() != self.precipitation_probability.len()
        {
            return Err("provider returned mismatched hourly arrays".into());
        }
        if self.time.len() > 16 * 25 {
            return Err("hourly data exceeds the supported forecast horizon".into());
        }
        let mut previous: Option<chrono::NaiveDateTime> = None;
        for i in 0..self.time.len() {
            let stamp = timestamp(&self.time[i])?;
            // Equal local times can occur when daylight-saving time ends.
            if previous.is_some_and(|p| stamp < p) {
                return Err("hourly timestamps must be ascending".into());
            }
            previous = Some(stamp);
            measurement(self.temperature_2m[i], None, None, "hourly temperature")?;
            measurement(
                self.precipitation_probability[i],
                Some(0.0),
                Some(100.0),
                "rain probability",
            )?;
        }
        Ok((0..self.time.len())
            .map(|i| Hour {
                time: self.time[i].clone(),
                temperature: self.temperature_2m[i],
                rain_probability: self.precipitation_probability[i],
            })
            .collect())
    }
}

fn measurement(
    value: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    name: &str,
) -> crate::Result<()> {
    if let Some(value) = value {
        if !value.is_finite() || min.is_some_and(|n| value < n) || max.is_some_and(|n| value > n) {
            return Err(format!("invalid {name} measurement"));
        }
    }
    Ok(())
}
pub(crate) fn calendar_date(value: &str) -> crate::Result<chrono::NaiveDate> {
    let date = chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| "invalid daily calendar date")?;
    if date.to_string() != value {
        return Err("daily dates must use canonical YYYY-MM-DD".into());
    }
    Ok(date)
}
pub(crate) fn timestamp(value: &str) -> crate::Result<chrono::NaiveDateTime> {
    let stamp = chrono::NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M")
        .map_err(|_| "invalid local weather timestamp")?;
    if stamp.format("%Y-%m-%dT%H:%M").to_string() != value {
        return Err("timestamps must use canonical YYYY-MM-DDTHH:MM".into());
    }
    Ok(stamp)
}
impl WeatherData {
    pub fn validate(&self) -> crate::Result<()> {
        if self.timezone.is_empty()
            || self.timezone.len() > 100
            || !self
                .timezone
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"/_-+".contains(&b))
        {
            return Err("invalid provider timezone".into());
        }
        if let Some(now) = &self.current {
            timestamp(&now.time)?;
            measurement(now.temperature_2m, None, None, "temperature")?;
            measurement(now.apparent_temperature, None, None, "apparent temperature")?;
            measurement(now.relative_humidity_2m, Some(0.0), Some(100.0), "humidity")?;
            measurement(now.wind_speed_10m, Some(0.0), None, "wind speed")?;
        }
        if let Some(daily) = &self.daily {
            daily.rows()?;
        }
        if let Some(hourly) = &self.hourly {
            hourly.rows()?;
        }
        Ok(())
    }
    pub fn forecast_coverage(&self, days: u8) -> crate::Result<()> {
        self.validate()?;
        let rows = self
            .daily
            .as_ref()
            .ok_or("provider returned no daily forecast")?
            .rows()?;
        if !(1..=16).contains(&days) || rows.len() != usize::from(days) {
            return Err("forecast does not cover the requested number of days".into());
        }
        if let Some(now) = &self.current {
            if calendar_date(&rows[0].date)? != timestamp(&now.time)?.date() {
                return Err("forecast period does not begin on the current local date".into());
            }
        }
        Ok(())
    }
    pub fn history_coverage(&self, range: crate::history::HistoryRange) -> crate::Result<()> {
        self.validate()?;
        let rows = self
            .daily
            .as_ref()
            .ok_or("provider returned no archive data")?
            .rows()?;
        if calendar_date(&rows[0].date)? != range.start
            || calendar_date(&rows[rows.len() - 1].date)? != range.end
        {
            return Err("archive does not cover the requested date range".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_observations_keep_missing_values_missing() {
        let data: WeatherData = serde_json::from_str(r#"{"timezone":"Europe/Istanbul","current":{"time":"2023-07-03T18:00","temperature_2m":24.5,"relative_humidity_2m":null,"weather_code":0}}"#).unwrap();
        let current = data.current.unwrap();
        assert_eq!(current.temperature_2m, Some(24.5));
        assert_eq!(current.relative_humidity_2m, None);
        assert_eq!(condition(current.weather_code), "Clear sky");
        assert_eq!(condition(Some(999)), "Unknown");
    }
}

#[cfg(test)]
mod daily_tests {
    use super::*;
    fn daily() -> Daily {
        serde_json::from_str(r#"{"time":["2023-07-01"],"temperature_2m_max":[25.0],"temperature_2m_min":[null],"precipitation_sum":[2.1],"weather_code":[61]}"#).unwrap()
    }
    #[test]
    fn daily_columns_become_rows_without_inventing_temperatures() {
        let rows = daily().rows().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].low, None);
        assert_eq!(rows[0].rain, Some(2.1));
    }
    #[test]
    fn rejects_ragged_daily_arrays() {
        let mut input = daily();
        input.temperature_2m_max.clear();
        assert!(input.rows().is_err());
    }
}

#[cfg(test)]
mod hourly_tests {
    use super::*;
    #[test]
    fn hourly_rows_preserve_time_and_probability() {
        let hourly: Hourly = serde_json::from_str(r#"{"time":["2023-12-01T18:00"],"temperature_2m":[null],"precipitation_probability":[75.0]}"#).unwrap();
        let rows = hourly.rows().unwrap();
        assert_eq!(rows[0].temperature, None);
        assert_eq!(rows[0].rain_probability, Some(75.0));
    }
    #[test]
    fn hourly_misalignment_is_rejected() {
        let hourly: Hourly = serde_json::from_str(r#"{"time":["2023-12-01T18:00"],"temperature_2m":[],"precipitation_probability":[75.0]}"#).unwrap();
        assert!(hourly.rows().is_err());
    }
}
