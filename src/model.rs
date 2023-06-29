use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherData {
    pub timezone: String,
    pub current: Option<Current>,
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
