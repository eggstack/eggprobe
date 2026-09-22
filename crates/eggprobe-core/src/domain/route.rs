//! Input routes and the redaction boundary used by reports.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A route requested by a plan. This type may contain credentials and is never
/// copied into a report.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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
            Self::Eggress(route) => crate::RouteSummary::Eggress {
                expression: redact_route_expression(&route.expression),
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
            Self::Eggress(route) => {
                formatter.write_str(&redact_route_expression(&route.expression))
            }
        }
    }
}

/// An input-only Eggress route expression.
#[derive(Clone, Eq, PartialEq, Serialize, Deserialize)]
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
        formatter.write_str(&redact_route_expression(self.0))
    }
}

fn redact_route_expression(expression: &str) -> String {
    let mut redacted = expression.to_owned();
    if let Some(scheme_end) = redacted.find("://") {
        let authority_start = scheme_end + 3;
        if let Some(authority_end_offset) = redacted[authority_start..].find('@') {
            let authority_end = authority_start + authority_end_offset;
            if let Some(password_separator) = redacted[authority_start..authority_end].find(':') {
                let separator = authority_start + password_separator;
                redacted.replace_range(separator + 1..authority_end, "[REDACTED]");
            }
        }
    }

    let mut output = String::with_capacity(redacted.len());
    let mut remainder = redacted.as_str();
    while let Some(separator) = remainder.find(['?', '&']) {
        let (prefix, query) = remainder.split_at(separator + 1);
        output.push_str(prefix);
        if let Some(end) = query.find('&') {
            let (parameter, tail) = query.split_at(end);
            output.push_str(&redact_parameter(parameter));
            remainder = tail;
        } else {
            output.push_str(&redact_parameter(query));
            remainder = "";
        }
    }
    output.push_str(remainder);
    output
}

fn redact_parameter(parameter: &str) -> String {
    let Some((key, _value)) = parameter.split_once('=') else {
        return parameter.to_owned();
    };
    let sensitive = matches!(
        key.to_ascii_lowercase().as_str(),
        "token" | "access_token" | "api_key" | "apikey" | "password" | "passwd" | "secret"
    );
    if sensitive {
        format!("{key}=[REDACTED]")
    } else {
        parameter.to_owned()
    }
}
