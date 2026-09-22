//! Pure renderers consuming canonical report types.

use std::fmt::Write;

use crate::ProbeReport;

/// Render a report for a human terminal without performing I/O or mutation.
#[must_use]
pub fn render_human(report: &ProbeReport) -> String {
    let mut output = format!(
        "{} {} — {}\nTarget: {}\nRoute: {:?}\n",
        report.tool.name,
        report.tool.version,
        format_status(&report.status),
        report.target.host,
        report.route,
    );
    for probe in &report.probes {
        writeln!(
            output,
            "- {:?}: {}",
            probe.kind,
            format_status(&probe.status)
        )
        .expect("writing to a String cannot fail");
        if let Some(error) = &probe.error {
            writeln!(output, "  error: {}", error.message)
                .expect("writing to a String cannot fail");
        }
        if let Some(evidence) = &probe.evidence {
            writeln!(output, "  evidence: {evidence:?}").expect("writing to a String cannot fail");
        }
        for unavailable in &probe.unavailable {
            writeln!(output, "  unavailable: {unavailable}")
                .expect("writing to a String cannot fail");
        }
    }
    output
}

fn format_status(status: &impl std::fmt::Debug) -> String {
    format!("{status:?}").to_ascii_lowercase()
}
