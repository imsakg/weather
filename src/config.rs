use crate::{cli::Options, location::Coordinates, Result};
use serde_derive::Deserialize;
use std::{io::Read, path::Path};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SavedLocation {
    pub city: Option<String>,
    pub country: Option<String>,
    pub coordinates: Option<Coordinates>,
}
impl SavedLocation {
    pub fn read(path: &Path) -> Result<Self> {
        let file =
            std::fs::File::open(path).map_err(|e| format!("cannot open location config: {e}"))?;
        let mut bytes = Vec::new();
        file.take(16_385)
            .read_to_end(&mut bytes)
            .map_err(|e| format!("cannot read config: {e}"))?;
        if bytes.len() > 16_384 {
            return Err("location config exceeds 16 KiB".into());
        }
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid location config: {e}"))
    }
    pub fn apply(self, args: &mut Options) -> Result<()> {
        if self.city.is_some() == self.coordinates.is_some() {
            return Err("saved config must specify either city or coordinates".into());
        }
        if let Some(city) = &self.city {
            crate::geocoding::search_url(city, self.country.as_deref())?;
        } else if self.country.is_some() {
            return Err("saved country requires a city".into());
        }
        if args.city.is_some() || args.latitude.is_some() || args.longitude.is_some() || args.locate
        {
            return Ok(());
        }
        if let Some(c) = self.coordinates {
            args.latitude = Some(c.latitude());
            args.longitude = Some(c.longitude());
        } else {
            args.city = self.city;
            args.country_code = self.country;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use structopt::StructOpt;
    #[test]
    fn saved_location_applies_only_without_an_explicit_location() {
        let defaults: SavedLocation =
            serde_json::from_str(r#"{"city":"Istanbul","country":"TR"}"#).unwrap();
        let mut args = crate::cli::Options::from_iter_safe(["weather"]).unwrap();
        defaults.clone().apply(&mut args).unwrap();
        assert_eq!(args.city.as_deref(), Some("Istanbul"));
        let mut explicit =
            crate::cli::Options::from_iter_safe(["weather", "London", "GB"]).unwrap();
        defaults.apply(&mut explicit).unwrap();
        assert_eq!(explicit.city.as_deref(), Some("London"));
        assert_eq!(explicit.country_code.as_deref(), Some("GB"));
    }
    #[test]
    fn conflicting_or_misspelled_defaults_are_not_silently_ignored() {
        assert!(serde_json::from_str::<SavedLocation>(r#"{"cty":"Paris"}"#).is_err());
        let defaults: SavedLocation =
            serde_json::from_str(r#"{"city":"Paris","coordinates":{"latitude":0,"longitude":0}}"#)
                .unwrap();
        let mut args = crate::cli::Options::from_iter_safe(["weather"]).unwrap();
        assert!(defaults.apply(&mut args).is_err());
    }
}
