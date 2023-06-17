use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::serde_utils;

pub const CONFIG_NAME: &str = "mega-cli-rs";
pub const DEFAULT_API_ORIGIN: &str = "https://g.api.mega.co.nz/";

/// The main configuration structure.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum Config {
    #[serde(rename = "1")]
    V1(V1Config),
}

/// The V1 configuration structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct V1Config {
    /// Authentication configuration.
    pub auth: AuthConfig,
    /// Configuration for the MEGA API client.
    pub client: ClientConfig,
}

/// Authentication configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    /// The serialized session string.
    pub session: Option<String>,
}

/// Configuration for the MEGA API client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientConfig {
    /// The API's origin.
    pub origin: Url,
    /// The number of allowed retries.
    pub max_retries: usize,
    /// The minimum amount of time between retries.
    #[serde(serialize_with = "serde_utils::duration::serialize")]
    #[serde(deserialize_with = "serde_utils::duration::deserialize")]
    pub min_retry_delay: Duration,
    /// The maximum amount of time between retries.
    #[serde(serialize_with = "serde_utils::duration::serialize")]
    #[serde(deserialize_with = "serde_utils::duration::deserialize")]
    pub max_retry_delay: Duration,
    /// The timeout duration to use for each request.
    #[serde(serialize_with = "serde_utils::duration::serialize_opt")]
    #[serde(deserialize_with = "serde_utils::duration::deserialize_opt")]
    pub timeout: Option<Duration>,
    /// Whether to use HTTPS for file downloads and uploads, instead of plain HTTP.
    ///
    /// Using plain HTTP for file transfers is fine because the file contents are already encrypted,
    /// making protocol-level encryption a bit redundant and potentially slowing down the transfer.
    pub https: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::V1(V1Config::default())
    }
}

impl Default for V1Config {
    fn default() -> Self {
        Self {
            auth: AuthConfig::default(),
            client: ClientConfig::default(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { session: None }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            origin: Url::parse(DEFAULT_API_ORIGIN).unwrap(),
            max_retries: 10,
            min_retry_delay: Duration::from_millis(10),
            max_retry_delay: Duration::from_secs(5),
            timeout: Some(Duration::from_secs(10)),
            https: false,
        }
    }
}
