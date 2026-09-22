//! Validated target input and safe report summary.

use std::{fmt, net::IpAddr};

use serde::{de, Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// A user-supplied host and optional service port.
#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TargetSpec {
    /// DNS name or IP literal.
    pub host: String,
    /// Optional port used by a port-oriented probe.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
}

impl<'de> Deserialize<'de> for TargetSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawTarget {
            host: String,
            #[serde(default)]
            port: Option<u16>,
        }

        let raw = RawTarget::deserialize(deserializer)?;
        Self::new(raw.host, raw.port).map_err(de::Error::custom)
    }
}

impl TargetSpec {
    /// Construct and validate a target without performing name resolution.
    ///
    /// # Errors
    ///
    /// Returns a [`TargetError`] when the host is syntactically invalid.
    pub fn new(host: impl Into<String>, port: Option<u16>) -> Result<Self, TargetError> {
        let target = Self {
            host: host.into(),
            port,
        };
        target.validate()?;
        Ok(target)
    }

    /// Validate the syntactic target boundary.
    ///
    /// # Errors
    ///
    /// Returns a [`TargetError`] when the host is syntactically invalid.
    pub fn validate(&self) -> Result<(), TargetError> {
        if self.host.is_empty() {
            return Err(TargetError::EmptyHost);
        }
        if self.host.len() > 253 {
            return Err(TargetError::HostTooLong);
        }
        if self.host.chars().any(char::is_whitespace) {
            return Err(TargetError::Whitespace);
        }
        if self.port == Some(0) {
            return Err(TargetError::InvalidPort);
        }
        if self.host.parse::<IpAddr>().is_ok() {
            return Ok(());
        }
        for label in self.host.trim_end_matches('.').split('.') {
            if label.is_empty() || label.len() > 63 {
                return Err(TargetError::InvalidLabel);
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(TargetError::InvalidLabel);
            }
            if !label
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '-')
            {
                return Err(TargetError::InvalidLabel);
            }
        }
        Ok(())
    }
}

impl fmt::Debug for TargetSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TargetSpec")
            .field("host", &self.host)
            .field("port", &self.port)
            .finish()
    }
}

/// Syntactic target validation failures.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TargetError {
    /// No host was supplied.
    #[error("target host must not be empty")]
    EmptyHost,
    /// Host names are bounded before any network operation.
    #[error("target host exceeds 253 characters")]
    HostTooLong,
    /// Whitespace is not valid in a host value.
    #[error("target host must not contain whitespace")]
    Whitespace,
    /// A host label is not valid DNS syntax.
    #[error("target host contains an invalid label")]
    InvalidLabel,
    /// Port zero is not a valid remote service target.
    #[error("target port must be between 1 and 65535")]
    InvalidPort,
}
