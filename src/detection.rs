use crate::{
    http::Http,
    location::{Coordinates, Location},
    Result,
};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IpLocation {
    #[serde(default)]
    pub error: bool,
    pub city: Option<String>,
    pub country_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}
impl IpLocation {
    pub fn into_location(self) -> Result<Location> {
        if self.error {
            return Err("IP location service declined the request".into());
        }
        let latitude = self.latitude.ok_or("IP location is missing latitude")?;
        let longitude = self.longitude.ok_or("IP location is missing longitude")?;
        Ok(Location {
            name: self
                .city
                .unwrap_or_else(|| "Approximate IP location".into()),
            country: self.country_code,
            coordinates: Coordinates::new(latitude, longitude)?,
        })
    }
}
/// Opt-in only: this discloses the caller's public IP to ipapi.co.
pub async fn detect(http: &Http) -> Result<Location> {
    http.get_json::<IpLocation>(
        "https://ipapi.co/json/"
            .parse()
            .map_err(|e| format!("invalid IP endpoint: {e}"))?,
    )
    .await?
    .into_location()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_ip_location_is_approximate_but_usable() {
        let raw: IpLocation = serde_json::from_str(
            r#"{"city":"Istanbul","country_code":"TR","latitude":41.0,"longitude":29.0}"#,
        )
        .unwrap();
        assert_eq!(raw.into_location().unwrap().name, "Istanbul");
    }
    #[test]
    fn provider_errors_and_missing_coordinates_are_rejected() {
        for json in [r#"{"error":true}"#, r#"{"city":"Unknown"}"#] {
            assert!(serde_json::from_str::<IpLocation>(json)
                .unwrap()
                .into_location()
                .is_err());
        }
    }
}
