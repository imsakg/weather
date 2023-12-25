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
        Some(0) => "Clear sky", Some(1) => "Mainly clear", Some(2) => "Partly cloudy", Some(3) => "Overcast",
        Some(45 | 48) => "Fog", Some(51 | 53 | 55) => "Drizzle", Some(56 | 57) => "Freezing drizzle",
        Some(61 | 63 | 65) => "Rain", Some(66 | 67) => "Freezing rain", Some(71 | 73 | 75 | 77) => "Snow",
        Some(80..=82) => "Rain showers", Some(85 | 86) => "Snow showers", Some(95) => "Thunderstorm",
        Some(96 | 99) => "Thunderstorm with hail", _ => "Unknown",
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
        let mut input = daily(); input.temperature_2m_max.clear();
        assert!(input.rows().is_err());
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
pub struct Day { pub date: String, pub high: Option<f64>, pub low: Option<f64>, pub rain: Option<f64>, pub code: Option<u16> }
impl Daily {
    pub fn rows(&self) -> crate::Result<Vec<Day>> {
        let len = self.time.len();
        if [self.temperature_2m_max.len(), self.temperature_2m_min.len(), self.precipitation_sum.len(), self.weather_code.len()].iter().any(|&n| n != len) {
            return Err("provider returned mismatched daily arrays".into());
        }
        Ok((0..len).map(|i| Day { date: self.time[i].clone(), high: self.temperature_2m_max[i], low: self.temperature_2m_min[i], rain: self.precipitation_sum[i], code: self.weather_code[i] }).collect())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hourly { pub time: Vec<String>, pub temperature_2m: Vec<Option<f64>>, pub precipitation_probability: Vec<Option<f64>> }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hour { pub time: String, pub temperature: Option<f64>, pub rain_probability: Option<f64> }
impl Hourly {
    pub fn rows(&self) -> crate::Result<Vec<Hour>> {
        if self.time.len() != self.temperature_2m.len() || self.time.len() != self.precipitation_probability.len() { return Err("provider returned mismatched hourly arrays".into()); }
        Ok((0..self.time.len()).map(|i| Hour { time:self.time[i].clone(), temperature:self.temperature_2m[i], rain_probability:self.precipitation_probability[i] }).collect())
    }
}
