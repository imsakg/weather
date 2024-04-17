use crate::{
    location::Location,
    model::{condition, WeatherData},
    units::Units,
};

pub fn number(value: Option<f64>) -> String {
    value
        .filter(|n| n.is_finite())
        .map_or_else(|| "—".into(), |n| format!("{n:.1}"))
}
pub fn current(location: &Location, data: &WeatherData, units: Units) -> String {
    let mut out = format!(
        "{}{} · {}\n",
        crate::text::terminal(&location.name),
        location
            .country
            .as_ref()
            .map_or(String::new(), |c| format!(", {}", crate::text::terminal(c))),
        crate::text::terminal(&data.timezone)
    );
    if let Some(now) = &data.current {
        out.push_str(&format!(
            "{} · {}\nTemperature: {} {} · feels like {} {}\nHumidity: {} % · wind: {} {}\n",
            crate::text::terminal(&now.time),
            condition(now.weather_code),
            number(now.temperature_2m),
            units.temperature_label(),
            number(now.apparent_temperature),
            units.temperature_label(),
            number(now.relative_humidity_2m),
            number(now.wind_speed_10m),
            units.wind_label()
        ));
    } else {
        out.push_str("Current observations unavailable\n");
    }
    out
}

pub fn daily(days: &[crate::model::Day], units: Units) -> String {
    let mut out = format!(
        "\n{:<12} {:>9} {:>9} {:>10}  Conditions\n",
        "Date",
        format!("High {}", units.temperature_label()),
        format!("Low {}", units.temperature_label()),
        format!("Rain {}", units.precipitation_label())
    );
    for day in days {
        out.push_str(&format!(
            "{:<12} {:>9} {:>9} {:>10}  {}\n",
            crate::text::terminal(&day.date),
            number(day.high),
            number(day.low),
            number(day.rain),
            condition(day.code)
        ));
    }
    out
}

pub fn history(location: &Location, data: &WeatherData, units: Units) -> crate::Result<String> {
    let days = data
        .daily
        .as_ref()
        .ok_or("archive response is missing daily data")?
        .rows()?;
    if days.is_empty() {
        return Err("archive returned no days".into());
    }
    Ok(format!("Historical weather · {} · {}\nOpen-Meteo reanalysis estimates; recent dates may be incomplete.\n{}", crate::text::terminal(&location.name), crate::text::terminal(&data.timezone), daily(&days, units)))
}

pub fn hourly(data: &WeatherData, units: Units, limit: u8) -> crate::Result<String> {
    let rows = data
        .hourly
        .as_ref()
        .ok_or("provider returned no hourly forecast")?
        .rows()?;
    let now = data.current.as_ref().map(|c| c.time.as_str()).unwrap_or("");
    let mut out = format!(
        "\nUpcoming hours · {}\n{:<18} {:>10} {:>12}\n",
        crate::text::terminal(&data.timezone),
        "Local time",
        units.temperature_label(),
        "Rain chance"
    );
    for hour in rows
        .iter()
        .filter(|r| r.time.as_str() >= now)
        .take(usize::from(limit.min(48)))
    {
        out.push_str(&format!(
            "{:<18} {:>10} {:>10} %\n",
            crate::text::terminal(&hour.time),
            number(hour.temperature),
            number(hour.rain_probability)
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_display_includes_units_and_missing_markers() {
        let data: WeatherData = serde_json::from_str(r#"{"timezone":"UTC","current":{"time":"2023-08-01T19:00","temperature_2m":12.5,"weather_code":61}}"#).unwrap();
        let location = Location::at(crate::location::Coordinates::new(0.0, 0.0).unwrap());
        let text = current(&location, &data, Units::Metric);
        assert!(text.contains("12.5 °C"));
        assert!(text.contains("Rain"));
        assert!(text.contains("Humidity: —"));
        assert!(text.contains("UTC"));
    }
}

#[cfg(test)]
mod daily_display_tests {
    use super::*;
    #[test]
    fn daily_table_has_dates_highs_lows_and_rain_units() {
        let days = vec![crate::model::Day {
            date: "2023-09-01".into(),
            high: Some(22.0),
            low: None,
            rain: Some(1.2),
            code: Some(61),
        }];
        let table = daily(&days, Units::Imperial);
        assert!(table.contains("2023-09-01"));
        assert!(table.contains("High °F"));
        assert!(table.contains("Rain in"));
        assert!(table.contains("22.0"));
        assert!(table.contains("—"));
    }
}

#[cfg(test)]
mod history_display_tests {
    use super::*;
    #[test]
    fn archive_output_is_not_mislabelled_as_a_current_observation() {
        let data: WeatherData = serde_json::from_str(r#"{"timezone":"UTC","daily":{"time":["2023-01-01"],"temperature_2m_max":[10.0],"temperature_2m_min":[2.0],"precipitation_sum":[null],"weather_code":[3]}}"#).unwrap();
        let location = Location::at(crate::location::Coordinates::new(0.0, 0.0).unwrap());
        let output = history(&location, &data, Units::Metric).unwrap();
        assert!(output.contains("Historical weather"));
        assert!(output.contains("reanalysis"));
        assert!(!output.contains("Current observations"));
        assert!(output.contains("2023-01-01"));
    }
}
