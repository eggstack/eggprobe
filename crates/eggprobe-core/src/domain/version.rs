//! Explicit schema and tool version values.

use std::{fmt, str::FromStr};

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// A pre-release machine-contract version, serialized as `"major.minor"`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SchemaVersion {
    /// Incompatible contract generation.
    pub major: u16,
    /// Additive contract generation within the major version.
    pub minor: u16,
}

impl JsonSchema for SchemaVersion {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "SchemaVersion".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^[0-9]+\\.[0-9]+$"
        })
    }
}

impl SchemaVersion {
    /// The historical contract version retained for compatibility fixtures.
    pub const INITIAL: Self = Self { major: 0, minor: 1 };

    /// The current contract version used by the active binary.
    pub const CURRENT: Self = Self { major: 0, minor: 3 };
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for SchemaVersion {
    type Err = VersionParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (major, minor) = value
            .split_once('.')
            .ok_or_else(|| VersionParseError::InvalidFormat(value.to_owned()))?;
        if major.is_empty() || minor.is_empty() || minor.contains('.') {
            return Err(VersionParseError::InvalidFormat(value.to_owned()));
        }
        Ok(Self {
            major: major
                .parse()
                .map_err(|_| VersionParseError::InvalidNumber(value.to_owned()))?,
            minor: minor
                .parse()
                .map_err(|_| VersionParseError::InvalidNumber(value.to_owned()))?,
        })
    }
}

impl Serialize for SchemaVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        value.parse().map_err(de::Error::custom)
    }
}

/// The Eggprobe binary/library version that produced a report.
#[derive(Clone, Debug, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
#[schemars(transparent)]
pub struct ToolVersion(String);

impl<'de> Deserialize<'de> for ToolVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

impl ToolVersion {
    /// Construct a tool version from a non-empty package version string.
    ///
    /// # Errors
    ///
    /// Returns [`VersionParseError::EmptyToolVersion`] for an empty value.
    pub fn new(value: impl Into<String>) -> Result<Self, VersionParseError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(VersionParseError::EmptyToolVersion);
        }
        Ok(Self(value))
    }

    /// Borrow the version text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ToolVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Errors returned when parsing explicit version values.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum VersionParseError {
    /// The value was not in `major.minor` form.
    #[error("invalid schema version format: {0:?}; expected major.minor")]
    InvalidFormat(String),
    /// A version component was not an unsigned integer.
    #[error("invalid schema version number: {0:?}")]
    InvalidNumber(String),
    /// Tool versions may not be empty.
    #[error("tool version must not be empty")]
    EmptyToolVersion,
}
