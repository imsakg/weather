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
    /// Latitude for an exact location (no geocoding)
    #[structopt(long, alias = "lat", requires = "longitude", conflicts_with = "city", allow_hyphen_values = true)]
    pub latitude: Option<f64>,
    /// Longitude for an exact location
    #[structopt(long, alias = "lon", requires = "latitude", conflicts_with = "city", allow_hyphen_values = true)]
    pub longitude: Option<f64>,
    /// Opt in to approximate public-IP location via ipapi.co
    #[structopt(long, conflicts_with_all = &["city", "latitude", "longitude"])]
    pub locate: bool,
    /// First historical date, YYYY-MM-DD (at most 366 days)
    #[structopt(long, requires = "end")]
    pub start: Option<String>,
    /// Last historical date, inclusive (archive can lag several days)
    #[structopt(long, requires = "start")]
    pub end: Option<String>,
    /// Add terminal temperature and precipitation charts
    #[structopt(long)]
    pub chart: bool,
    /// Show the next 1..48 available hourly points; 0 hides them
    #[structopt(long, default_value = "0", parse(try_from_str = parse_hours))]
    pub hourly: u8,
}
fn parse_days(value: &str) -> Result<u8> {
    let days = value.parse::<u8>().map_err(|_| "days must be an integer from 1 to 16")?;
    if !(1..=16).contains(&days) { return Err("days must be between 1 and 16".into()); }
    Ok(days)
}
impl Options {
    pub fn coordinates(&self) -> Result<Option<crate::location::Coordinates>> {
        match (self.latitude, self.longitude) {
            (Some(lat), Some(lon)) => crate::location::Coordinates::new(lat, lon).map(Some),
            (None, None) => Ok(None),
            _ => Err("provide both latitude and longitude".into()),
        }
    }
    pub fn history_range(&self) -> Result<Option<crate::history::HistoryRange>> {
        match (&self.start, &self.end) {
            (Some(start),Some(end)) => crate::history::HistoryRange::new(start, end, chrono::Utc::now().date_naive()).map(Some),
            (None,None) => Ok(None),
            _ => Err("provide both --start and --end".into()),
        }
    }
    pub fn validate(&self) -> Result<()> {
        self.history_range()?;
        if self.start.is_some() && self.hourly > 0 { return Err("hourly display is not available for archive requests".into()); }
        if self.coordinates()?.is_some() || self.locate { return Ok(()); }
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

#[cfg(test)]
mod coordinate_tests {
    use super::*;
    #[test]
    fn signed_coordinates_work_without_geocoding() {
        let args = Options::from_iter_safe(["weather", "--latitude", "-33.86", "--longitude", "151.2"]).unwrap();
        assert!(args.validate().is_ok());
        assert_eq!(args.coordinates().unwrap().unwrap().latitude(), -33.86);
    }
    #[test]
    fn incomplete_or_conflicting_location_is_rejected() {
        assert!(Options::from_iter_safe(["weather", "--latitude", "2"]).is_err());
        assert!(Options::from_iter_safe(["weather", "London", "--latitude", "2", "--longitude", "3"]).is_err());
        let args = Options::from_iter_safe(["weather", "--latitude", "200", "--longitude", "0"]).unwrap();
        assert!(args.validate().is_err());
    }
}

fn parse_hours(value: &str) -> Result<u8> {
    value.parse::<u8>().ok().filter(|n| *n <= 48).ok_or_else(|| "hourly must be an integer from 0 to 48".into())
}
