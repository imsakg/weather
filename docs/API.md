# API and data contract

The client targets the keyless public APIs documented by
[Open-Meteo forecast](https://open-meteo.com/en/docs),
[geocoding](https://open-meteo.com/en/docs/geocoding-api), and
[historical weather](https://open-meteo.com/en/docs/historical-weather-api).
Provider availability, terms and quotas can change independently of this client.

## Requests

### Named locations

`GET https://geocoding-api.open-meteo.com/v1/search`

- `name`: trimmed city name, URL-encoded as a query value, 1–200 characters.
- `count=10`, `language=en`, `format=json`.
- Optional `countryCode`: two-letter uppercase country code.

Responses contain `results` with names, coordinates and optional `country_code`.
An absent/empty result list is an actionable location-not-found error. Results
are filtered locally by country when one was requested. The first remaining
match is used; coordinates avoid this ambiguity.

### Current and forecast weather

`GET https://api.open-meteo.com/v1/forecast`

Common fields are `latitude`, `longitude`, `timezone=auto`,
`temperature_unit=celsius|fahrenheit`, `wind_speed_unit=kmh|mph` and
`precipitation_unit=mm|inch`.

- `current=temperature_2m,relative_humidity_2m,apparent_temperature,weather_code,wind_speed_10m`
- `daily=temperature_2m_max,temperature_2m_min,precipitation_sum,weather_code`
- `hourly=temperature_2m,precipitation_probability`
- `forecast_days=1..16`

Current fields are optional measurements. Daily and hourly values arrive as
parallel arrays keyed by local ISO dates/timestamps. Values may be `null`.
Weather-code descriptions follow WMO codes; unfamiliar codes display `Unknown`.
Precipitation probability and relative humidity are percentages, not fractions.
Wind is a speed and precipitation totals cannot be negative. Hourly output
selects values at or after the supplied current local timestamp, not blindly
from midnight. Daylight-saving changes can repeat a local hourly timestamp;
daily dates must remain strictly ordered.

### Historical weather

`GET https://archive-api.open-meteo.com/v1/archive`

Uses the same coordinate/unit/timezone and daily fields, with inclusive
`start_date` and `end_date`. There is no current or hourly request in archive
mode. Daily timestamps must match the requested interval. Archived data are
reanalysis estimates; recent days can be unavailable or have null measurements.
The client permits dates since 1940, limits requests to 366 days and does not
substitute invented observations for gaps.

### IP location

`GET https://ipapi.co/json/`, only with explicit `--locate`.

The response must provide valid latitude and longitude. City/country labels are
optional. Provider-declared errors or missing coordinates are errors. This is an
approximate public-IP lookup, not GPS, and discloses the caller's IP to a separate
provider. It is never a fallback for a city lookup failure.

## Exported report schema 1

```json
{
  "schema_version": 1,
  "kind": "forecast",
  "units": "metric",
  "location": {
    "name": "Example location",
    "country": "TR",
    "coordinates": { "latitude": 41.0, "longitude": 29.0 }
  },
  "data": {
    "timezone": "Europe/Istanbul",
    "current": null,
    "daily": {
      "time": ["2024-03-01"],
      "temperature_2m_max": [null],
      "temperature_2m_min": [null],
      "precipitation_sum": [null],
      "weather_code": [null]
    },
    "hourly": null
  }
}
```

This is a schema illustration, not a live observation. `kind` is `forecast` or
`history`; `units` is `metric` or `imperial`. The report's embedded units and kind
remain authoritative during replay. `daily` must contain at least one aligned
row. `current` and `hourly` may be absent or null; requesting hourly rendering
without a trustworthy current timestamp is an error. Unknown schema versions
are rejected. The checked-in forecast fixture contains deterministic test data.

Cache files wrap a report with `version`, `key` and `fetched_at` Unix seconds;
this wrapper is not the export format. Cached reports must also match the
request's units, kind, explicit coordinates and requested date/length coverage.
A digest filename alone does not authenticate a cache entry. Cache notices and
freshness warnings are stderr metadata, not JSON fields.

## Failure behavior

- Non-success HTTP statuses are rejected before JSON parsing; 429 reports rate
  limiting. No redirects are followed or implicit retries attempted.
- Request/connect timeouts and streaming length checks bound downloads.
- Invalid JSON, ragged arrays, impossible measurements, invalid/out-of-order
  dates and unsupported report schemas fail explicitly.
- Null values remain null in JSON and gaps in tables/charts/exports.
- Diagnostic output belongs on stderr; report output must not be partially
  emitted before a validation/rendering failure.
