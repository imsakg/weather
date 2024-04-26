# Architecture

## Data flow

1. `cli` parses options and validates location, date and numeric ranges.
2. `config` optionally supplies a location, never overriding explicit choices.
3. `app` selects report replay, an eligible cache entry, or a live provider call.
4. `http` bounds each response and enforces timeouts and HTTP status handling.
5. `geocoding`/`detection` resolve locations; `api` builds forecast/archive queries.
6. `model` decodes provider columns into validated rows; `report` is the common
   versioned envelope for fetched, cached and imported weather.
7. `display`, `chart`, `summary` and `export` consume the same report. Output
   formatting must not change whether a report is considered valid.

`main` owns stdout/stderr and exit behavior. Core functions return errors instead
of exiting. Unit tests exercise transformations; socket tests exercise the real
HTTP client against loopback servers; CLI tests run the compiled executable.
Public-provider checks are intentionally not part of the deterministic suite.

## Trust boundaries

Treat providers, cache files and input reports as untrusted data. All enter via
bounded reads. Coordinates must be finite and within geographic limits. Parallel
arrays must align; null measurements remain null. Calendar values, ordering,
physical ranges, report kinds and cache/request consistency are checked before a
report is displayed or accepted from disk. A cache key is not proof that the
stored contents match it.

Terminal text strips control and bidi-direction characters and is length-bounded.
Unicode names remain readable. JSON uses proper serializer escaping. CSV quotes
special fields, leaves null numeric cells empty and neutralizes spreadsheet
formula prefixes in text cells. Failures must not leave partial successful-looking
stdout. None of these formats substitutes zero for missing weather.

## Cache design

Caching is explicit through `--cache-dir`. Request keys include normalized request
parameters, not display switches. SHA-256 filenames avoid path traversal through
city names. Each cache document stores its schema version, full request key,
fetch time and report. A temporary file in the same directory is synchronized
and atomically persisted; the previous target is not truncated in place.

The clock/TTL policy takes an explicit timestamp, making boundary tests
deterministic. Future cache timestamps are rejected. Old entries are only usable
with `--offline`, and their age is disclosed on stderr. Online cache corruption
causes a warning and fresh request, never a stale fallback. Input replay does not
read cache or use the network.

## Limits and deliberate omissions

- HTTP and report/cache inputs: 2 MiB; saved configuration: 16 KiB.
- Connect timeout: 5 seconds; total request timeout: 15 seconds; no redirects.
- Forecast: 1–16 days; historical range: 1–366 inclusive days.
- Hourly display: at most 48 upcoming rows; it requires a current reference time.
- Sparkline width: at most 120 columns; precipitation bars: at most 80.
- No automatic IP location, retries, background refresh, account tracking,
  paid-provider credentials, telemetry, server component or secret storage.
- Cache age is on stderr rather than inside report schema 1; capture that stream
  when freshness matters. Offline data is not suitable for safety decisions.

## Maintenance gates

Run the exact commands in `.github/workflows/ci.yml` locally before handoff:
format check, strict all-target Clippy, locked tests and a locked release build.
The workflow is configured for Linux, macOS and Windows; local macOS evidence is
not proof that remote CI ran. Commit timestamp reconstruction is documented in
`DEVELOPMENT.md` and must not be represented as historical execution evidence.
