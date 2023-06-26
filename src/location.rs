use serde_derive::{Deserialize, Serialize};
use crate::Result;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "RawCoordinates")]
pub struct Coordinates { latitude: f64, longitude: f64 }
#[derive(Deserialize)]
struct RawCoordinates { latitude: f64, longitude: f64 }
impl TryFrom<RawCoordinates> for Coordinates {
    type Error = String;
    fn try_from(raw: RawCoordinates) -> Result<Self> { Self::new(raw.latitude, raw.longitude) }
}
impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self> {
        if !latitude.is_finite() || !longitude.is_finite() || !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
            return Err("latitude must be -90..90 and longitude -180..180 (finite)".into());
        }
        Ok(Self { latitude, longitude })
    }
    pub fn latitude(self) -> f64 { self.latitude }
    pub fn longitude(self) -> f64 { self.longitude }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Location { pub name: String, pub country: Option<String>, pub coordinates: Coordinates }
impl Location {
    pub fn at(coordinates: Coordinates) -> Self {
        Self { name: format!("{:.4}, {:.4}", coordinates.latitude(), coordinates.longitude()), country: None, coordinates }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn coordinates_reject_nonfinite_and_out_of_range_values() {
        assert!(Coordinates::new(90.0, -180.0).is_ok());
        for (lat, lon) in [(90.1, 0.0), (0.0, 180.1), (f64::NAN, 0.0), (0.0, f64::INFINITY)] {
            assert!(Coordinates::new(lat, lon).is_err());
        }
    }
    #[test]
    fn deserialization_cannot_bypass_coordinate_validation() {
        assert!(serde_json::from_str::<Coordinates>(r#"{"latitude":91,"longitude":0}"#).is_err());
    }
}
