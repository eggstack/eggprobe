//! Thin CLI surface over `eggprobe-core`.

use clap::Parser;

use eggprobe_core::ProbeReport;

/// Command-line parser. Network commands are intentionally deferred until
/// concrete probe implementations exist.
#[derive(Debug, Parser)]
#[command(name = "eggprobe", version, about = "JSON-first network diagnostics")]
pub struct Cli {}

/// Parse the supported foundation command line.
#[must_use]
pub fn parse() -> Cli {
    Cli::parse()
}

/// Render a canonical report through the core renderer.
#[must_use]
pub fn render_report(report: &ProbeReport) -> String {
    eggprobe_core::render::render_human(report)
}
