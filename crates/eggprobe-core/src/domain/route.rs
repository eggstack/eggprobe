//! Input routes and the redaction boundary used by reports.

use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A route requested by a plan. This type may contain credentials and is never
/// copied into a report.
#[derive(Clone, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RouteSpec {
    /// Direct target access.
    Direct,
    /// An Eggress route expression, retained only at the plan boundary.
    Eggress(EggressRoute),
}

impl RouteSpec {
    /// Produce a report-safe route summary without exposing the input value.
    #[must_use]
    pub fn summary(&self) -> crate::RouteSummary {
        match self {
            Self::Direct => crate::RouteSummary::Direct,
            Self::Eggress(_route) => crate::RouteSummary::Eggress {
                expression: "<redacted>".to_owned(),
            },
        }
    }
}

impl fmt::Debug for RouteSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Direct => formatter.write_str("Direct"),
            Self::Eggress(route) => formatter
                .debug_tuple("Eggress")
                .field(&Redacted(&route.expression))
                .finish(),
        }
    }
}

impl fmt::Display for RouteSpec {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Direct => formatter.write_str("direct"),
            Self::Eggress(_) => formatter.write_str("eggress(<redacted>)"),
        }
    }
}

/// An input-only Eggress route expression.
#[derive(Clone, Eq, JsonSchema, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EggressRoute {
    /// Raw route expression supplied by the caller.
    pub expression: String,
}

impl fmt::Debug for EggressRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EggressRoute")
            .field("expression", &Redacted(&self.expression))
            .finish()
    }
}

struct Redacted<'a>(&'a str);

impl fmt::Debug for Redacted<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = self.0;
        formatter.write_str("<redacted>")
    }
}
