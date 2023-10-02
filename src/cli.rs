use structopt::StructOpt;
use crate::{units::Units, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "weather", about = "Current weather and forecasts from Open-Meteo (no API key)")]
pub struct Options {
    /// City name; quote names containing spaces
    pub city: Option<String>,
    /// Two-letter ISO country code to disambiguate a city
    pub country_code: Option<String>,
    /// Forecast length, from 1 to 16 days
    #[structopt(long, default_value = "7", parse(try_from_str = parse_days))]
    pub days: u8,
    /// metric or imperial
    #[structopt(long, default_value = "metric")]
    pub units: Units,
}
fn parse_days(value: &str) -> Result<u8> {
    let days = value.parse::<u8>().map_err(|_| "days must be an integer from 1 to 16")?;
    if !(1..=16).contains(&days) { return Err("days must be between 1 and 16".into()); }
    Ok(days)
}
impl Options {
    pub fn validate(&self) -> Result<()> {
        let city = self.city.as_deref().ok_or("provide a city; see weather --help")?;
        crate::geocoding::search_url(city, self.country_code.as_deref())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use structopt::StructOpt;
    #[test]
    fn accepts_legacy_positionals_and_explicit_unit_flags() {
        let args = Options::from_iter_safe(["weather", "London", "GB", "--days", "3", "--units", "imperial"]).unwrap();
        assert_eq!(args.city.as_deref(), Some("London"));
        assert_eq!(args.days, 3);
        assert_eq!(args.units, Units::Imperial);
        assert!(args.validate().is_ok());
    }
    #[test]
    fn invalid_ranges_and_missing_location_fail_before_network() {
        assert!(Options::from_iter_safe(["weather", "London", "--days", "17"]).is_err());
        assert!(Options::from_iter_safe(["weather"]).unwrap().validate().is_err());
    }
}
