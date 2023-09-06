use crate::{location::Location, model::{condition, WeatherData}, units::Units};

pub fn number(value: Option<f64>) -> String {
    value.filter(|n| n.is_finite()).map_or_else(|| "—".into(), |n| format!("{n:.1}"))
}
pub fn current(location: &Location, data: &WeatherData, units: Units) -> String {
    let mut out = format!("{}{} · {}\n", location.name, location.country.as_ref().map_or(String::new(), |c| format!(", {c}")), data.timezone);
    if let Some(now) = &data.current {
        out.push_str(&format!("{} · {}\nTemperature: {} {} · feels like {} {}\nHumidity: {} % · wind: {} {}\n", now.time, condition(now.weather_code),
            number(now.temperature_2m), units.temperature_label(), number(now.apparent_temperature), units.temperature_label(),
            number(now.relative_humidity_2m), number(now.wind_speed_10m), units.wind_label()));
    } else { out.push_str("Current observations unavailable\n"); }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_display_includes_units_and_missing_markers() {
        let data: WeatherData = serde_json::from_str(r#"{"timezone":"UTC","current":{"time":"2023-08-01T19:00","temperature_2m":12.5,"weather_code":61}}"#).unwrap();
        let location = Location::at(crate::location::Coordinates::new(0.0,0.0).unwrap());
        let text = current(&location, &data, Units::Metric);
        assert!(text.contains("12.5 °C"));
        assert!(text.contains("Rain"));
        assert!(text.contains("Humidity: —"));
        assert!(text.contains("UTC"));
    }
}
