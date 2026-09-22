//! Public timing values with one canonical JSON unit.

use serde::{Deserialize, Serialize};

use super::error::DiagnosticStage;

/// A duration represented as integer microseconds in JSON.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurationMicros(pub u64);

impl DurationMicros {
    /// Construct a duration from seconds.
    #[must_use]
    pub const fn from_secs(seconds: u64) -> Self {
        Self(seconds.saturating_mul(1_000_000))
    }

    /// Construct a duration from microseconds.
    #[must_use]
    pub const fn from_micros(micros: u64) -> Self {
        Self(micros)
    }

    /// Return the integer microsecond value.
    #[must_use]
    pub const fn as_micros(self) -> u64 {
        self.0
    }
}

/// Timing for one probe, including only phases that were truthfully observed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Timing {
    /// Total measured duration.
    pub total: DurationMicros,
    /// Ordered phase measurements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phases: Vec<PhaseTiming>,
}

/// A named phase duration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhaseTiming {
    /// Boundary at which the duration was measured.
    pub stage: DiagnosticStage,
    /// Measured phase duration.
    pub duration: DurationMicros,
}
