use reqwest::Url;
use crate::{http::Http, location::Coordinates, model::WeatherData, units::Units, Result};

pub const DAILY_FIELDS: &str = "temperature_2m_max,temperature_2m_min,precipitation_sum,weather_code";
pub fn forecast_url(coordinates: Coordinates, units: Units, days: u8) -> Result<Url> {
    if !(1..=16).contains(&days) { return Err("forecast days must be between 1 and 16".into()); }
    let mut url = Url::parse("https://api.open-meteo.com/v1/forecast").map_err(|e| e.to_string())?;
    add_common(&mut url, coordinates, units);
    url.query_pairs_mut().append_pair("current", "temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m")
        .append_pair("daily", DAILY_FIELDS).append_pair("forecast_days", &days.to_string());
    Ok(url)
}
pub fn add_common(url: &mut Url, coordinates: Coordinates, units: Units) {
    url.query_pairs_mut().append_pair("latitude", &coordinates.latitude().to_string()).append_pair("longitude", &coordinates.longitude().to_string())
        .append_pair("temperature_unit", units.temperature_parameter()).append_pair("wind_speed_unit", units.wind_parameter())
        .append_pair("precipitation_unit", units.precipitation_parameter()).append_pair("timezone", "auto");
}
pub async fn forecast(http: &Http, coordinates: Coordinates, units: Units, days: u8) -> Result<WeatherData> {
    let data: WeatherData = http.get_json(forecast_url(coordinates, units, days)?).await?;
    data.daily.as_ref().ok_or("provider returned no daily forecast")?.rows()?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn forecast_query_includes_coordinates_units_and_local_time() {
        let url = forecast_url(Coordinates::new(41.0, 29.0).unwrap(), Units::Imperial, 7).unwrap();
        let query: std::collections::HashMap<_,_> = url.query_pairs().into_owned().collect();
        assert_eq!(query["forecast_days"], "7");
        assert_eq!(query["temperature_unit"], "fahrenheit");
        assert_eq!(query["precipitation_unit"], "inch");
        assert_eq!(query["timezone"], "auto");
        assert!(forecast_url(Coordinates::new(0.0,0.0).unwrap(), Units::Metric, 0).is_err());
        assert!(forecast_url(Coordinates::new(0.0,0.0).unwrap(), Units::Metric, 17).is_err());
    }
}
