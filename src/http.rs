use std::time::Duration;
use reqwest::{Client, Url};
use serde::de::DeserializeOwned;
use crate::Result;

pub const MAX_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
#[derive(Clone)]
pub struct Http { client: Client }
impl Http {
    pub fn new() -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(15)).connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none()).user_agent(concat!("weather/", env!("CARGO_PKG_VERSION")))
            .build().map_err(|e| format!("cannot initialize HTTP: {}", e.without_url()))?;
        Ok(Self { client })
    }
    pub async fn get_json<T: DeserializeOwned>(&self, url: Url) -> Result<T> {
        let mut response = self.client.get(url).send().await.map_err(|e| format!("request failed: {}", e.without_url()))?;
        let status = response.status();
        if status.as_u16() == 429 { return Err("provider rate limit reached; retry later".into()); }
        if !status.is_success() { return Err(format!("provider returned HTTP {}", status.as_u16())); }
        if response.content_length().is_some_and(|n| n > MAX_RESPONSE_BYTES as u64) { return Err("provider response is too large".into()); }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(|e| format!("response interrupted: {}", e.without_url()))? {
            if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES { return Err("provider response is too large".into()); }
            bytes.extend_from_slice(&chunk);
        }
        serde_json::from_slice(&bytes).map_err(|e| format!("invalid provider JSON: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{io::{Read, Write}, net::TcpListener, thread};
    fn serve(status: &str, body: &str) -> reqwest::Url {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap()).parse().unwrap();
        let reply = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
            let mut input = [0; 8192]; let _ = stream.read(&mut input);
            stream.write_all(reply.as_bytes()).unwrap();
        });
        url
    }
    #[tokio::test]
    async fn reads_json_over_a_real_loopback_connection() {
        let value: serde_json::Value = Http::new().unwrap().get_json(serve("200 OK", r#"{"ok":true}"#)).await.unwrap();
        assert_eq!(value["ok"], true);
    }
    #[tokio::test]
    async fn rate_limits_are_actionable_errors() {
        let error = Http::new().unwrap().get_json::<serde_json::Value>(serve("429 Too Many Requests", "{}")).await.unwrap_err();
        assert!(error.contains("rate limit"));
    }
}
