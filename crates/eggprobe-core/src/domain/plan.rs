//! Typed execution intent.

use std::net::IpAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use super::{route::RouteSpec, target::TargetSpec, timing::DurationMicros, version::SchemaVersion};

/// A complete, versioned request for an Eggprobe execution.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbePlan {
    /// Machine contract version of this plan.
    pub schema_version: SchemaVersion,
    /// Target selected by the caller.
    pub target: TargetSpec,
    /// Requested route, which may contain input-only credentials.
    pub route: RouteSpec,
    /// Ordered probe requests.
    pub probes: Vec<ProbeSpec>,
    /// Execution limits and repetition policy.
    #[serde(default)]
    pub execution: ExecutionPolicy,
    /// Deferred assertion declarations retained as typed plan data.
    #[serde(default)]
    pub assertions: Vec<AssertionSpec>,
}

impl ProbePlan {
    /// Validate cross-field authority and execution bounds before execution.
    ///
    /// # Errors
    ///
    /// Returns a [`PlanValidationError`] when the schema, limits, or target
    /// authority is invalid.
    pub fn validate(&self) -> Result<(), PlanValidationError> {
        if self.schema_version != SchemaVersion::CURRENT {
            return Err(PlanValidationError::UnsupportedSchema(self.schema_version));
        }
        if self.execution.deadline.as_micros() == 0 {
            return Err(PlanValidationError::ZeroDeadline);
        }
        if self.execution.repetitions == 0 {
            return Err(PlanValidationError::ZeroRepetitions);
        }
        if self.execution.retries > 0 {
            return Err(PlanValidationError::UnsupportedRetries(
                self.execution.retries,
            ));
        }
        for assertion in &self.assertions {
            if let AssertionKind::HttpStatusRange { min, max } = &assertion.assertion {
                if min > max {
                    return Err(PlanValidationError::InvalidAssertionRange {
                        min: *min,
                        max: *max,
                    });
                }
            }
        }
        for probe in &self.probes {
            match probe {
                ProbeSpec::Tcp { port } | ProbeSpec::Tls { port, .. } if *port == 0 => {
                    return Err(PlanValidationError::InvalidPort);
                }
                ProbeSpec::Http { url, .. } => {
                    let authority = HttpAuthority::parse(url)?;
                    if !same_host(&authority.host, &self.target.host) {
                        return Err(PlanValidationError::TargetMismatch {
                            expected: self.target.host.clone(),
                            actual: authority.host,
                        });
                    }
                    if let Some(expected) = self.target.port {
                        let actual = authority.port;
                        if expected != actual {
                            return Err(PlanValidationError::PortMismatch { expected, actual });
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// Cross-field plan validation failures.
#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum PlanValidationError {
    /// The plan schema is not supported by this binary.
    #[error("unsupported plan schema version: {0}")]
    UnsupportedSchema(SchemaVersion),
    /// The outer deadline must be finite and non-zero.
    #[error("execution deadline must be greater than zero")]
    ZeroDeadline,
    /// At least one execution is required.
    #[error("execution repetitions must be greater than zero")]
    ZeroRepetitions,
    /// Retries are reserved for a future explicit attempt contract.
    #[error("retries are not supported by schema 0.3 (requested {0})")]
    UnsupportedRetries(u32),
    /// An inclusive HTTP assertion range cannot have its lower bound above its upper bound.
    #[error("HTTP status assertion minimum {min} exceeds maximum {max}")]
    InvalidAssertionRange {
        /// Inclusive lower status bound.
        min: u16,
        /// Inclusive upper status bound.
        max: u16,
    },
    /// A port-oriented probe cannot use port zero.
    #[error("probe port must be between 1 and 65535")]
    InvalidPort,
    /// An HTTP URL host disagrees with the authoritative plan target.
    #[error("HTTP target host {actual:?} disagrees with plan target {expected:?}")]
    TargetMismatch {
        /// Authoritative plan target host.
        expected: String,
        /// Conflicting HTTP URL host.
        actual: String,
    },
    /// An HTTP URL port disagrees with the explicit plan port.
    #[error("HTTP target port {actual} disagrees with plan port {expected}")]
    PortMismatch {
        /// Authoritative plan target port.
        expected: u16,
        /// Conflicting HTTP URL port.
        actual: u16,
    },
    /// The HTTP URL is not an absolute URL with an authority.
    #[error("invalid HTTP URL: {0}")]
    InvalidHttpUrl(String),
}

struct HttpAuthority {
    host: String,
    port: u16,
}

impl HttpAuthority {
    fn parse(raw: &str) -> Result<Self, PlanValidationError> {
        let url =
            Url::parse(raw).map_err(|_| PlanValidationError::InvalidHttpUrl(raw.to_owned()))?;
        let default_port = match url.scheme() {
            "http" => 80,
            "https" => 443,
            _ => return Err(PlanValidationError::InvalidHttpUrl(raw.to_owned())),
        };
        if !url.username().is_empty() || url.password().is_some() {
            return Err(PlanValidationError::InvalidHttpUrl(raw.to_owned()));
        }
        let host = url
            .host_str()
            .filter(|host| !host.is_empty())
            .ok_or_else(|| PlanValidationError::InvalidHttpUrl(raw.to_owned()))?
            .trim_start_matches('[')
            .trim_end_matches(']')
            .to_owned();
        let port = url.port().unwrap_or(default_port);
        if port == 0 {
            return Err(PlanValidationError::InvalidHttpUrl(raw.to_owned()));
        }
        Ok(Self { host, port })
    }
}

fn same_host(left: &str, right: &str) -> bool {
    match (left.parse::<IpAddr>(), right.parse::<IpAddr>()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left
            .trim_end_matches('.')
            .eq_ignore_ascii_case(right.trim_end_matches('.')),
    }
}

/// A probe request with no execution implementation in the foundation milestone.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum ProbeSpec {
    /// Resolve the target name.
    Dns,
    /// Establish a direct or routed byte-stream connection.
    Tcp {
        /// Port to connect to.
        port: u16,
    },
    /// Perform a standalone TLS handshake.
    Tls {
        /// Port to connect to.
        port: u16,
        /// Optional logical TLS server name.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        server_name: Option<String>,
    },
    /// Perform an HTTP request through the eventual HTTP owner.
    Http {
        /// Request URL.
        url: String,
        /// HTTP method, defaulting to GET.
        #[serde(default = "default_http_method")]
        method: String,
    },
}

fn default_http_method() -> String {
    "GET".to_owned()
}

/// Execution limits that are meaningful before network execution exists.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPolicy {
    /// Outer deadline for one execution, in integer microseconds.
    pub deadline: DurationMicros,
    /// Number of declared repetitions.
    pub repetitions: u32,
    /// Explicit retry count per attempt.
    pub retries: u32,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            deadline: DurationMicros::from_secs(30),
            repetitions: 1,
            retries: 0,
        }
    }
}

/// A typed assertion declaration evaluated against completed report evidence.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssertionSpec {
    /// Stable caller-provided assertion identifier.
    pub id: String,
    /// Typed assertion operation.
    pub assertion: AssertionKind,
}

/// Supported, non-scripted assertion operations.
#[derive(Clone, Debug, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum AssertionKind {
    /// Require an observed HTTP response status in an inclusive range.
    HttpStatusRange {
        /// Inclusive lower status bound.
        min: u16,
        /// Inclusive upper status bound.
        max: u16,
    },
    /// Require an observed HTTP protocol version string.
    RequiredHttpVersion {
        /// Required protocol value, for example `HTTP/2.0`.
        version: String,
    },
    /// Require an observed negotiated ALPN value.
    RequiredAlpn {
        /// Required ALPN token.
        value: String,
    },
    /// Require an observed TLS version string.
    RequiredTlsVersion {
        /// Required TLS version value.
        version: String,
    },
    /// Require total probe timing to remain below a microsecond threshold.
    MaxTotalMicros {
        /// Maximum allowed total duration in microseconds.
        micros: u64,
    },
}
