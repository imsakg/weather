use std::time::{SystemTime, UNIX_EPOCH};
use crate::{api, cache::Cache, cli::Options, geocoding, http::Http, location::Location, report::{Report, ReportKind}, Result};

pub async fn load(args: &Options) -> Result<Report> {
    args.validate()?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| "system clock is before the Unix epoch")?.as_secs();
    let key = args.request_key();
    let cache = args.cache_dir.clone().map(Cache::new);
    if !args.refresh {
        if let Some(cache) = &cache {
            match cache.lookup(&key, now, args.cache_ttl, args.offline) {
                Ok(Some(hit)) => {
                    eprintln!("{}cached weather · {} seconds old", if hit.stale { "STALE " } else { "" }, hit.age_seconds);
                    return Ok(hit.report);
                },
                Ok(None) => {},
                Err(error) if args.offline => return Err(error),
                Err(error) => eprintln!("Ignoring unusable cache: {error}"),
            }
        }
    }
    if args.offline { return Err("offline mode requires a usable cached report".into()); }
    let http = Http::new()?;
    let location = if args.locate {
        eprintln!("IP location is approximate; contacting ipapi.co at your request.");
        crate::detection::detect(&http).await?
    } else if let Some(coordinates) = args.coordinates()? { Location::at(coordinates) }
    else { geocoding::resolve(&http, args.city.as_deref().unwrap_or_default(), args.country_code.as_deref()).await? };
    let (data, kind) = if let Some(range) = args.history_range()? {
        (api::archive(&http, location.coordinates, args.units, range).await?, ReportKind::History)
    } else { (api::forecast(&http, location.coordinates, args.units, args.days).await?, ReportKind::Forecast) };
    let report = Report::new(location, args.units, kind, data);
    if let Some(cache) = cache {
        if let Err(error) = cache.store(&key, now, &report) { eprintln!("Could not save cache: {error}"); }
    }
    Ok(report)
}
