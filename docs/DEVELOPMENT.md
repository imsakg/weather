# Development plan and history provenance

The additional commit sequence is a reconstructed implementation history, not
contemporaneous activity from 2023–2024. Author and committer timestamps were
assigned at the repository owner's request. The original root commit is preserved.

## Scope

Build a usable Rust weather CLI with a keyless Open-Meteo integration, named and
coordinate locations, opt-in IP location, current/hourly/daily forecasts, archived
weather, terminal charts, machine-readable exports, cache and saved defaults.

## Acceptance

- Deterministic unit and loopback HTTP tests; no public network in `cargo test`.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- Exercise real forecast, geocoding and archive endpoints separately.
- No embedded credentials, implicit IP lookup, pushes, or destructive data writes.

Keep API transport, validated models and presentation independent. Each feature
should land with its own focused tests. Missing provider values remain missing,
not fabricated zeroes. Caches must be bounded, versioned and explicitly stale
when used offline. Forecast and archive schemas must be checked before rendering.
