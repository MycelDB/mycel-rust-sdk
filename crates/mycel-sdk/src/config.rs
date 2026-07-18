use std::{env, time::Duration, time::SystemTime};

use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub const DEFAULT_ADDR: &str = "127.0.0.1:9091";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub addr: String,

    pub username: String,
    pub password: String,
    pub access_token: String,
    pub access_token_expire_time: Option<SystemTime>,
    pub refresh_token: String,
    pub refresh_before: Option<Duration>,
    pub call_timeout: Option<Duration>,
    pub primary_follow: PrimaryFollowPolicy,

    pub tls: bool,
    pub tls_ca_file: String,
    pub tls_server_name: String,
    pub tls_insecure_skip_verify: bool,
    pub tls_client_cert_file: String,
    pub tls_client_key_file: String,

    pub client_name: String,
    pub client_version: String,
    pub platform: String,
    pub device_label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryFollowPolicy {
    pub enabled: bool,
    pub retry_reads: bool,
    pub retry_unsafe: bool,
    pub max_redirects: usize,
}

impl Default for PrimaryFollowPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            retry_reads: true,
            retry_unsafe: false,
            max_redirects: 1,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: DEFAULT_ADDR.to_string(),
            username: String::new(),
            password: String::new(),
            access_token: String::new(),
            access_token_expire_time: None,
            refresh_token: String::new(),
            refresh_before: None,
            call_timeout: None,
            primary_follow: PrimaryFollowPolicy::default(),
            tls: false,
            tls_ca_file: String::new(),
            tls_server_name: String::new(),
            tls_insecure_skip_verify: false,
            tls_client_cert_file: String::new(),
            tls_client_key_file: String::new(),
            client_name: "mycel-rust-sdk".to_string(),
            client_version: String::new(),
            platform: "rust".to_string(),
            device_label: String::new(),
        }
    }
}

impl Config {
    pub fn addr(&self) -> &str {
        let addr = self.addr.trim();
        if addr.is_empty() {
            DEFAULT_ADDR
        } else {
            addr
        }
    }

    pub fn endpoint_uri(&self) -> String {
        let addr = self.addr();
        if addr.starts_with("http://") || addr.starts_with("https://") {
            addr.to_string()
        } else if self.tls {
            format!("https://{addr}")
        } else {
            format!("http://{addr}")
        }
    }

    pub fn from_env() -> Self {
        Self {
            addr: first_non_empty([
                env::var("MYCELD_GRPC_ADDR").ok(),
                Some(DEFAULT_ADDR.to_string()),
            ]),
            username: env::var("MYCEL_USERNAME").unwrap_or_default(),
            password: env::var("MYCEL_PASSWORD").unwrap_or_default(),
            access_token: env::var("MYCEL_ACCESS_TOKEN").unwrap_or_default(),
            access_token_expire_time: env::var("MYCEL_ACCESS_TOKEN_EXPIRE_TIME")
                .ok()
                .and_then(|v| parse_time(&v)),
            refresh_token: env::var("MYCEL_REFRESH_TOKEN").unwrap_or_default(),
            refresh_before: env::var("MYCEL_REFRESH_BEFORE")
                .ok()
                .and_then(|v| parse_duration(&v)),
            call_timeout: env::var("MYCEL_CALL_TIMEOUT")
                .ok()
                .and_then(|v| parse_duration(&v)),
            primary_follow: PrimaryFollowPolicy::default(),
            tls: env::var("MYCELD_TLS")
                .map(|v| parse_bool(&v))
                .unwrap_or(false),
            tls_ca_file: env::var("MYCELD_TLS_CA_FILE").unwrap_or_default(),
            tls_server_name: env::var("MYCELD_TLS_SERVER_NAME").unwrap_or_default(),
            tls_insecure_skip_verify: env::var("MYCELD_TLS_INSECURE_SKIP_VERIFY")
                .map(|v| parse_bool(&v))
                .unwrap_or(false),
            tls_client_cert_file: env::var("MYCELD_TLS_CLIENT_CERT_FILE").unwrap_or_default(),
            tls_client_key_file: env::var("MYCELD_TLS_CLIENT_KEY_FILE").unwrap_or_default(),
            client_name: first_non_empty([
                env::var("MYCEL_CLIENT_NAME").ok(),
                Some("mycel-rust-sdk".to_string()),
            ]),
            client_version: env::var("MYCEL_CLIENT_VERSION").unwrap_or_default(),
            platform: first_non_empty([
                env::var("MYCEL_CLIENT_PLATFORM").ok(),
                Some("rust".to_string()),
            ]),
            device_label: env::var("MYCEL_CLIENT_DEVICE_LABEL").unwrap_or_default(),
        }
    }
}

fn first_non_empty<const N: usize>(values: [Option<String>; N]) -> String {
    values
        .into_iter()
        .flatten()
        .map(|v| v.trim().to_string())
        .find(|v| !v.is_empty())
        .unwrap_or_default()
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "t" | "true" | "y" | "yes" | "on"
    )
}

fn parse_duration(value: &str) -> Option<Duration> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(ms) = value.strip_suffix("ms") {
        return ms.parse::<u64>().ok().map(Duration::from_millis);
    }
    if let Some(s) = value.strip_suffix('s') {
        return s.parse::<u64>().ok().map(Duration::from_secs);
    }
    if let Some(m) = value.strip_suffix('m') {
        return m.parse::<u64>().ok().map(|v| Duration::from_secs(v * 60));
    }
    if let Some(h) = value.strip_suffix('h') {
        return h
            .parse::<u64>()
            .ok()
            .map(|v| Duration::from_secs(v * 60 * 60));
    }
    None
}

fn parse_time(value: &str) -> Option<SystemTime> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    OffsetDateTime::parse(value, &Rfc3339)
        .ok()
        .map(SystemTime::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoint_uri_adds_scheme() {
        let cfg = Config {
            addr: "127.0.0.1:9999".into(),
            ..Default::default()
        };
        assert_eq!(cfg.endpoint_uri(), "http://127.0.0.1:9999");
        let cfg = Config { tls: true, ..cfg };
        assert_eq!(cfg.endpoint_uri(), "https://127.0.0.1:9999");
    }

    #[test]
    fn duration_parser_matches_go_style_values() {
        assert_eq!(parse_duration("5s"), Some(Duration::from_secs(5)));
        assert_eq!(parse_duration("250ms"), Some(Duration::from_millis(250)));
        assert_eq!(parse_duration("2m"), Some(Duration::from_secs(120)));
        assert_eq!(parse_duration("bad"), None);
    }

    #[test]
    fn time_parser_accepts_rfc3339() {
        assert!(parse_time("2026-07-03T12:00:00Z").is_some());
        assert!(parse_time("bad").is_none());
    }
}
