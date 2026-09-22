//! Typed execution intent.

use serde::{Deserialize, Serialize};

use super::{route::RouteSpec, target::TargetSpec, timing::DurationMicros, version::SchemaVersion};

/// A complete, versioned request for an Eggprobe execution.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

/// A probe request with no execution implementation in the foundation milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
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

/// An assertion declaration. Evaluation belongs to a later milestone.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssertionSpec {
    /// Stable caller-provided assertion identifier.
    pub id: String,
    /// Human-readable deferred expression.
    pub description: String,
}
