//! Real loopback sockets, deterministic provider fixtures; no public API calls.
use std::{io::{Read, Write}, net::TcpListener, thread};
use weather::{http::{Http, MAX_RESPONSE_BYTES}, geocoding::SearchResults};

fn serve(response: String) -> (reqwest::Url, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap()).parse().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();
        let mut request = vec![0; 16384];
        let count = stream.read(&mut request).unwrap();
        let request = String::from_utf8_lossy(&request[..count]).into_owned();
        let _ = stream.write_all(response.as_bytes());
        request
    });
    (url, handle)
}
fn response(status: &str, body: &str) -> String {
    format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len())
}
#[tokio::test]
async fn encoded_city_reaches_the_server_and_decodes_a_location() {
    let (mut url, server) = serve(response("200 OK", r#"{"results":[{"name":"São Paulo","latitude":-23.55,"longitude":-46.63,"country_code":"BR"}]}"#));
    let query = weather::geocoding::search_url("São Paulo & coast", Some("BR")).unwrap();
    url.set_query(query.query());
    let result = Http::new().unwrap().get_json::<SearchResults>(url).await.unwrap();
    assert_eq!(result.select(Some("BR")).unwrap().name, "São Paulo");
    assert!(server.join().unwrap().contains("name=S%C3%A3o+Paulo+%26+coast"));
}
#[tokio::test]
async fn malformed_json_is_distinct_from_http_failure() {
    let (url, server) = serve(response("200 OK", "not JSON"));
    assert!(Http::new().unwrap().get_json::<serde_json::Value>(url).await.unwrap_err().contains("invalid provider JSON"));
    server.join().unwrap();
}
#[tokio::test]
async fn upstream_failures_do_not_deserialize_as_weather() {
    let (url, server) = serve(response("503 Service Unavailable", "{}"));
    assert!(Http::new().unwrap().get_json::<serde_json::Value>(url).await.unwrap_err().contains("HTTP 503"));
    server.join().unwrap();
}
#[tokio::test]
async fn redirects_are_not_followed() {
    let (url, server) = serve("HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:1/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".into());
    assert!(Http::new().unwrap().get_json::<serde_json::Value>(url).await.unwrap_err().contains("HTTP 302"));
    server.join().unwrap();
}
#[tokio::test]
async fn oversized_content_length_is_rejected_before_body_read() {
    let (url, server) = serve(format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", MAX_RESPONSE_BYTES + 1));
    assert!(Http::new().unwrap().get_json::<serde_json::Value>(url).await.unwrap_err().contains("too large"));
    server.join().unwrap();
}
#[tokio::test]
async fn oversized_chunked_bodies_are_also_bounded() {
    let body = "x".repeat(MAX_RESPONSE_BYTES + 1);
    let response = format!("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{:x}\r\n{}\r\n0\r\n\r\n", body.len(), body);
    let (url, server) = serve(response);
    assert!(Http::new().unwrap().get_json::<serde_json::Value>(url).await.unwrap_err().contains("too large"));
    server.join().unwrap();
}
