# Weather

A Rust terminal weather app: current conditions, up to 16 forecast days,
upcoming hourly weather, historical reanalysis, charts, and JSON/CSV exports.
Uses [Open-Meteo](https://open-meteo.com/) without an API key.

## Install

Install a current stable Rust toolchain, then run:

```sh
cargo build --release --locked
./target/release/weather --help
# Or install a local copy into ~/.cargo/bin:
cargo install --path . --locked
```

The application uses HTTPS and the platform certificate store. No API key or
secret configuration is required. Older versions used OpenWeather; the old
`OPENWEATHER_API_KEY` variable is no longer used.

## Current and daily forecasts

```sh
weather London GB
weather "São Paulo" BR --days 3
weather Istanbul TR --units imperial --chart
weather --latitude -33.86 --longitude 151.2 --days 5
weather Istanbul TR --hourly 12
```

- Countries use two-letter ISO codes. When multiple matches remain, the first
  geocoding match is selected; use coordinates for precision.
- Units are `metric` (°C, km/h, mm) or `imperial` (°F, mph, inches).
- `--days` accepts 1–16. `--hourly` accepts 0–48 (0 hides the hourly table).
  Hourly output starts at or after the provider's current local timestamp and
  requires a usable current timestamp. Late-day requests may return fewer rows.
- Times use the provider's local timezone. Missing measurements display as `—`
  or a chart gap, never as a made-up zero.
- Forecasts and current values are weather-model estimates, not safety alerts.

### Opt-in location detection

```sh
weather --locate --days 3
```

`--locate` contacts [ipapi.co](https://ipapi.co/) using your public IP. Detection
is approximate and can be affected by VPNs. It never runs implicitly; explicit
coordinates do not contact the geocoding or IP-location providers.

## Historical weather

```sh
weather Istanbul TR --start 2023-07-01 --end 2023-07-07 --chart
weather --latitude 51.51 --longitude -0.13 --start 2024-02-29 --end 2024-02-29
```

Dates must be canonical `YYYY-MM-DD` calendar dates, ordered and not in the future
(UTC date). The inclusive range is limited to 366 days and starts no earlier than
1940-01-01. Historical values are **reanalysis estimates**, not archived forecasts
or individual station readings. Recent dates can lag by several days or contain
missing values. Hourly display is only supported for forecast requests.

## Cache and offline use

Caching is opt-in. Files include the selected location, coordinates and weather;
choose an appropriate private directory.

```sh
weather Istanbul TR --cache-dir "$HOME/.cache/weather"
weather Istanbul TR --cache-dir "$HOME/.cache/weather" --offline --chart
weather Istanbul TR --cache-dir "$HOME/.cache/weather" --refresh
weather Istanbul TR --cache-dir "$HOME/.cache/weather" --cache-ttl 1800
```

The default TTL is 900 seconds; the maximum is 86400. The key includes location,
units, forecast length and historical dates, but not presentation flags. Fresh
cache hits avoid all network calls. `--refresh` bypasses cache reads. `--offline`
never contacts any provider and explicitly permits old entries; cache age and a
`STALE` warning go to stderr. No stale fallback occurs on an online request.
Cache errors do not prevent an online fetch; offline misses and corrupt entries
are errors. Atomic writes keep interrupted updates from truncating prior data.

## Saved locations

Copy `config.example.json` to a path of your choice, then:

```sh
weather --config ./config.example.json --days 3
weather London GB --config ./config.example.json
```

The file accepts either `{"city":"Istanbul","country":"TR"}` or
`{"coordinates":{"latitude":41.0,"longitude":29.0}}`. It is read only when
`--config` is supplied. An explicit city, coordinate pair or `--locate` overrides
the saved location. Unknown fields and ambiguous saved locations are rejected.

## Machine-readable output and replay

```sh
weather Istanbul TR --json > report.json
weather Istanbul TR --csv > daily.csv
weather --input report.json --chart
# Network-free demo using a deterministic test fixture, not live weather:
cargo run -- --input tests/fixtures/forecast.json --chart --hourly 2
```

JSON includes `schema_version`, `kind`, `units`, `location` and provider-shaped
`data`; missing values remain `null`. JSON and CSV contain no progress text on
stdout. Cache/privacy notices and errors go to stderr. Preserve that stream when
recording cached results, because cache age is not a JSON field.

CSV contains daily rows with explicit unit columns, CRLF record endings, quoted
text where needed and empty missing-value cells. Formula-like text is prefixed
with an apostrophe for spreadsheet safety. JSON/CSV cannot be combined with
charts. `--input` replays an exported version-1 report without network or cache
access; its embedded units and forecast/history kind are authoritative.

## Development and tests

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run -- --input tests/fixtures/forecast.json --json
```

Tests use deterministic data and real loopback HTTP sockets; `cargo test` does
not call public weather services. Live smoke checks are a separate operation.
Failures exit nonzero. Invalid arguments and missing locations fail before any
network request. HTTP calls have bounded response sizes, connection/request
timeouts, and explicit status checks; redirects are not followed.

See [development scope and history provenance](docs/DEVELOPMENT.md). The added
commit timestamps are a reconstructed sequence requested by the repository
owner, not a claim that this implementation was authored in 2023–2024. The
original root commit is unchanged.

## Data attribution and terms

Weather and geocoding data are provided by Open-Meteo and its upstream datasets.
Use is subject to the provider's [terms](https://open-meteo.com/en/terms) and
[data attribution requirements](https://open-meteo.com/en/docs). The free endpoint
is intended for non-commercial use and has limits. Commercial deployments need
an appropriate provider agreement; this client does not manage paid API keys.
IP location is a separate service with separate terms.
