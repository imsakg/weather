//! A small, testable weather client. Networking is separate from presentation.
pub mod units;
pub type Result<T> = std::result::Result<T, String>;
pub mod location;
pub mod model;
pub mod http;
pub mod geocoding;
pub mod api;
pub mod display;
pub mod cli;
pub mod detection;
pub mod history;
pub mod chart;
pub mod report;
pub mod cache;
