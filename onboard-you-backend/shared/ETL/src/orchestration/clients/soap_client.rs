//! Shared HTTP / SOAP client abstractions for orchestration layer
//!
//! Houses protocol-level clients (SOAP, REST, etc.) that connectors and
//! egress capabilities rely on. By centralising transport here, every
//! capability gets a single, mockable seam for network I/O.

use onboard_you_models::{Error, Result};

// ───────────────────────────────────────────────────────────────────────────
// SOAP Client Trait
// ───────────────────────────────────────────────────────────────────────────

/// Trait abstracting the HTTP POST call for SOAP services.
///
/// Inject a mock in tests or swap in a different HTTP backend without
/// changing any connector logic.
pub trait SoapClient: Send + Sync {
    /// Send a SOAP POST request and return the raw XML response body.
    fn post_soap(&self, endpoint: &str, envelope: &str) -> Result<String>;
}

// ───────────────────────────────────────────────────────────────────────────
// Production ReqwestSoapClient
// ───────────────────────────────────────────────────────────────────────────

/// Simple tag-extractor used only for error reporting in the client layer.
fn extract_tag_value(xml: &str, tag: &str) -> String {
    for prefix in &["bsvc:", "wd:", ""] {
        let open = format!("<{}{}>", prefix, tag);
        let close = format!("</{}{}>", prefix, tag);
        if let Some(start) = xml.find(&open) {
            let content_start = start + open.len();
            if let Some(end) = xml[content_start..].find(&close) {
                let text = xml[content_start..content_start + end]
                    .split('<')
                    .next()
                    .unwrap_or("")
                    .trim();
                if !text.is_empty() {
                    return text.to_string();
                }
            }
        }
    }
    String::new()
}

/// Production HTTP client that sends SOAP requests via `reqwest` (blocking).
///
/// Uses `rustls-tls` so the binary ships without a system OpenSSL dependency.
#[derive(Debug, Clone)]
pub struct ReqwestSoapClient;

impl SoapClient for ReqwestSoapClient {
    fn post_soap(&self, endpoint: &str, envelope: &str) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(false)
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| Error::IngestionError(format!("HTTP client error: {}", e)))?;

        let response = client
            .post(endpoint)
            .header("Content-Type", "text/xml; charset=utf-8")
            .header("SOAPAction", "")
            .body(envelope.to_owned())
            .send()
            .map_err(|e| {
                Error::IngestionError(format!("SOAP request to '{}' failed: {}", endpoint, e))
            })?;

        let status = response.status();
        let body = response.text().map_err(|e| {
            Error::IngestionError(format!("Failed to read SOAP response body: {}", e))
        })?;

        if !status.is_success() {
            let fault_msg = extract_tag_value(&body, "faultstring");
            let detail = if fault_msg.is_empty() {
                body.chars().take(500).collect::<String>()
            } else {
                fault_msg
            };
            return Err(Error::IngestionError(format!(
                "SOAP service returned HTTP {}: {}",
                status, detail
            )));
        }

        Ok(body)
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Tests
// ───────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct StubClient(String);

    impl SoapClient for StubClient {
        fn post_soap(&self, _endpoint: &str, _envelope: &str) -> Result<String> {
            Ok(self.0.clone())
        }
    }

    struct FailClient;

    impl SoapClient for FailClient {
        fn post_soap(&self, _endpoint: &str, _envelope: &str) -> Result<String> {
            Err(Error::IngestionError("connection refused".into()))
        }
    }

    #[test]
    fn test_stub_client_returns_body() {
        let c = StubClient("<ok/>".into());
        let res = c.post_soap("http://x", "<env/>").unwrap();
        assert_eq!(res, "<ok/>");
    }

    #[test]
    fn test_fail_client_returns_error() {
        let c = FailClient;
        assert!(c.post_soap("http://x", "<env/>").is_err());
    }

    #[test]
    fn test_extract_tag_value_faultstring() {
        let xml = r#"<env:Body><env:Fault><faultstring>Not Authorized</faultstring></env:Fault></env:Body>"#;
        assert_eq!(extract_tag_value(xml, "faultstring"), "Not Authorized");
    }

    #[test]
    fn test_extract_tag_value_missing() {
        assert_eq!(extract_tag_value("<a>1</a>", "missing"), "");
    }
}
