use std::{fs::File, io::{Read, Write}, path::PathBuf};
use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::{http::MAX_RESPONSE_BYTES, report::Report, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cached { pub version: u8, pub key: String, pub fetched_at: u64, pub report: Report }
pub struct Cache { root: PathBuf }
impl Cache {
    pub fn new(root: PathBuf) -> Self { Self { root } }
    pub fn path_for(&self, key: &str) -> PathBuf { self.root.join(format!("{:x}.json", Sha256::digest(key.as_bytes()))) }
    pub fn load(&self, key: &str) -> Result<Option<Cached>> {
        let file = match File::open(self.path_for(key)) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("cannot read cache: {e}")),
        };
        let mut bytes = Vec::new();
        file.take(MAX_RESPONSE_BYTES as u64 + 1).read_to_end(&mut bytes).map_err(|e| format!("cannot read cache: {e}"))?;
        if bytes.len() > MAX_RESPONSE_BYTES { return Err("cache entry is too large".into()); }
        let entry: Cached = serde_json::from_slice(&bytes).map_err(|e| format!("invalid cache JSON: {e}"))?;
        if entry.version != 1 || entry.report.schema_version != 1 || entry.key != key { return Err("cache schema or request key mismatch".into()); }
        Ok(Some(entry))
    }
    pub fn store(&self, key: &str, fetched_at: u64, report: &Report) -> Result<()> {
        let entry = Cached { version: 1, key: key.into(), fetched_at, report: report.clone() };
        let bytes = serde_json::to_vec(&entry).map_err(|e| e.to_string())?;
        if bytes.len() > MAX_RESPONSE_BYTES { return Err("report is too large to cache".into()); }
        std::fs::create_dir_all(&self.root).map_err(|e| format!("cannot create cache directory: {e}"))?;
        let mut temporary = tempfile::NamedTempFile::new_in(&self.root).map_err(|e| format!("cannot create cache file: {e}"))?;
        temporary.write_all(&bytes).map_err(|e| format!("cannot write cache: {e}"))?;
        temporary.as_file().sync_all().map_err(|e| format!("cannot sync cache: {e}"))?;
        temporary.persist(self.path_for(key)).map_err(|e| format!("cannot replace cache: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn report() -> Report {
        Report::new(crate::location::Location::at(crate::location::Coordinates::new(1.0,2.0).unwrap()), crate::units::Units::Metric, crate::report::ReportKind::Forecast, serde_json::from_str(r#"{"timezone":"UTC"}"#).unwrap())
    }
    #[test]
    fn cache_round_trip_and_key_isolation() {
        let dir = tempfile::tempdir().unwrap(); let cache = Cache::new(dir.path().to_path_buf());
        assert!(cache.load("London").unwrap().is_none());
        cache.store("London", 1000, &report()).unwrap();
        assert_eq!(cache.load("London").unwrap().unwrap().fetched_at, 1000);
        assert!(cache.load("Paris").unwrap().is_none());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
        assert!(cache.path_for("../secrets").parent() == Some(dir.path()));
    }
    #[test]
    fn corrupt_cache_does_not_become_a_weather_report() {
        let dir = tempfile::tempdir().unwrap(); let cache = Cache::new(dir.path().to_path_buf());
        std::fs::write(cache.path_for("a"), "bad json").unwrap();
        assert!(cache.load("a").is_err());
    }
}
