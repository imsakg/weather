use reqwest::Url;
use serde_derive::Deserialize;
use crate::{http::Http, location::{Coordinates, Location}, Result};

pub fn search_url(city: &str, country: Option<&str>) -> Result<Url> {
    if city.trim().is_empty() || city.chars().count() > 200 { return Err("city must contain 1..200 characters".into()); }
    let mut url = Url::parse("https://geocoding-api.open-meteo.com/v1/search").map_err(|e| e.to_string())?;
    url.query_pairs_mut().append_pair("name", city.trim()).append_pair("count", "10").append_pair("language", "en").append_pair("format", "json");
    if let Some(country) = country {
        if country.len() != 2 || !country.bytes().all(|b| b.is_ascii_alphabetic()) { return Err("country must be a two-letter ISO code".into()); }
        url.query_pairs_mut().append_pair("countryCode", &country.to_ascii_uppercase());
    }
    Ok(url)
}
#[derive(Debug, Deserialize)]
pub struct Place { pub name: String, pub latitude: f64, pub longitude: f64, pub country_code: Option<String> }
#[derive(Debug, Deserialize)]
pub struct SearchResults { #[serde(default)] pub results: Vec<Place> }
impl SearchResults {
    pub fn select(self, country: Option<&str>) -> Result<Location> {
        let place = self.results.into_iter().find(|p| country.map_or(true, |code| p.country_code.as_deref().is_some_and(|v| v.eq_ignore_ascii_case(code))))
            .ok_or_else(|| "location not found; try a larger city or explicit coordinates".to_string())?;
        Ok(Location { name: place.name, country: place.country_code, coordinates: Coordinates::new(place.latitude, place.longitude)? })
    }
}
pub async fn resolve(http: &Http, city: &str, country: Option<&str>) -> Result<Location> {
    http.get_json::<SearchResults>(search_url(city, country)?).await?.select(country)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_values_are_encoded_and_country_is_validated() {
        let url = search_url("São Paulo & coast", Some("br")).unwrap();
        let pairs: std::collections::HashMap<_,_> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["name"], "São Paulo & coast");
        assert_eq!(pairs["countryCode"], "BR");
        assert!(search_url("", None).is_err());
        assert!(search_url("London", Some("UK!")).is_err());
    }
    #[test]
    fn empty_search_results_are_not_an_unchecked_index() {
        let response: SearchResults = serde_json::from_str("{}").unwrap();
        assert!(response.select(None).is_err());
    }
    #[test]
    fn filters_ambiguous_places_by_country() {
        let response: SearchResults = serde_json::from_str(r#"{"results":[{"name":"Paris","latitude":33.6,"longitude":-95.5,"country_code":"US"},{"name":"Paris","latitude":48.8,"longitude":2.3,"country_code":"FR"}]}"#).unwrap();
        assert_eq!(response.select(Some("fr")).unwrap().country.as_deref(), Some("FR"));
    }
}
