//! Thin Clap adapter over `eggprobe-core`.

use std::{
    fs,
    io::{self, Read},
    path::PathBuf,
};

use clap::{Args, Parser, Subcommand};
use eggprobe_core::{
    evaluate_assertions, exit_code, AssertionKind, AssertionSpec, ExecutionPolicy, ProbeEngine,
    ProbePlan, ProbeReport, ProbeSpec, RouteSpec, SchemaVersion, TargetPolicy, TargetSpec,
};

const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_BATCH_SIZE: usize = 256;

/// Top-level command line.
#[derive(Debug, Parser)]
#[command(name = "eggprobe", version, about = "JSON-first network diagnostics")]
pub struct Cli {
    /// Selected subcommand.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Supported diagnostic operations.
#[derive(Debug, Subcommand)]
pub enum Command {
    Dns(PrimitiveArgs),
    Tcp(PrimitiveArgs),
    Tls(TlsArgs),
    Http(HttpArgs),
    Proxy(ProxyArgs),
    Check(CheckArgs),
    Run(RunArgs),
    Compare(CompareArgs),
}

/// Common primitive controls.
#[derive(Clone, Debug, Args)]
pub struct PrimitiveArgs {
    pub target: String,
    #[arg(short, long)]
    pub port: Option<u16>,
    #[arg(long)]
    pub via: Option<String>,
    #[arg(long, default_value_t = 30_000)]
    pub timeout_ms: u64,
    #[arg(long)]
    pub strict_target: bool,
    #[arg(long)]
    pub json: bool,
}
/// TLS controls.
#[derive(Clone, Debug, Args)]
pub struct TlsArgs {
    #[command(flatten)]
    pub primitive: PrimitiveArgs,
    #[arg(long)]
    pub server_name: Option<String>,
}
/// HTTP controls.
#[derive(Clone, Debug, Args)]
pub struct HttpArgs {
    pub url: String,
    #[arg(long)]
    pub via: Option<String>,
    #[arg(long, default_value = "GET")]
    pub method: String,
    #[arg(long, default_value_t = 30_000)]
    pub timeout_ms: u64,
    #[arg(long)]
    pub strict_target: bool,
    #[arg(long)]
    pub json: bool,
}
/// Explicit proxy route controls.
#[derive(Clone, Debug, Args)]
pub struct ProxyArgs {
    pub target: String,
    #[arg(short, long)]
    pub port: u16,
    #[arg(long)]
    pub via: String,
    #[arg(long, default_value_t = 30_000)]
    pub timeout_ms: u64,
    #[arg(long)]
    pub json: bool,
}
/// Composite check controls.
#[derive(Clone, Debug, Args)]
pub struct CheckArgs {
    pub target: String,
    #[arg(short, long)]
    pub port: u16,
    #[arg(long)]
    pub url: Option<String>,
    #[arg(long, default_value_t = 200)]
    pub expect_status_min: u16,
    #[arg(long, default_value_t = 299)]
    pub expect_status_max: u16,
    #[arg(long, default_value_t = 30_000)]
    pub timeout_ms: u64,
    #[arg(long)]
    pub via: Option<String>,
    #[arg(long)]
    pub strict_target: bool,
    #[arg(long)]
    pub json: bool,
}
/// Plan-file and batch controls.
#[derive(Clone, Debug, Args)]
pub struct RunArgs {
    pub input: PathBuf,
    #[arg(long)]
    pub ndjson: bool,
    #[arg(long, default_value_t = 4)]
    pub concurrency: usize,
    #[arg(long)]
    pub fail_fast: bool,
}
/// Repeated comparison controls.
#[derive(Clone, Debug, Args)]
pub struct CompareArgs {
    pub direct: PathBuf,
    pub routed: PathBuf,
    #[arg(long, default_value_t = 1)]
    pub repeat: u32,
    #[arg(long)]
    pub json: bool,
}

/// Parse the command line.
#[must_use]
pub fn parse() -> Cli {
    Cli::parse()
}

/// Execute a parsed command and return its stable process code and output.
///
/// # Errors
///
/// Returns a bounded invocation, plan, or serialization error.
pub async fn execute(cli: Cli) -> Result<(i32, String), String> {
    let Some(command) = cli.command else {
        return Ok((0, "Run `eggprobe --help` for commands.\n".into()));
    };
    match command {
        Command::Dns(args) => run_single(plan_for_dns(&args)?, args.json, args.strict_target).await,
        Command::Tcp(args) => run_single(plan_for_tcp(&args)?, args.json, args.strict_target).await,
        Command::Tls(args) => {
            run_single(
                plan_for_tls(&args)?,
                args.primitive.json,
                args.primitive.strict_target,
            )
            .await
        }
        Command::Http(args) => {
            run_single(plan_for_http(&args)?, args.json, args.strict_target).await
        }
        Command::Proxy(args) => run_single(plan_for_proxy(&args)?, args.json, false).await,
        Command::Check(args) => {
            run_single(plan_for_check(&args)?, args.json, args.strict_target).await
        }
        Command::Run(args) => run_input(args).await,
        Command::Compare(args) => run_compare(args).await,
    }
}

async fn run_single(plan: ProbePlan, json: bool, strict: bool) -> Result<(i32, String), String> {
    plan.validate().map_err(|error| error.to_string())?;
    let engine = ProbeEngine {
        target_policy: if strict {
            TargetPolicy::Strict
        } else {
            TargetPolicy::AllowPrivate
        },
    };
    let mut report = engine.execute(plan.clone()).await;
    let findings = evaluate_assertions(&report, &plan.assertions);
    report.findings = findings;
    Ok((exit_code(&report) as i32, render(&report, json)))
}

async fn run_input(args: RunArgs) -> Result<(i32, String), String> {
    if args.concurrency == 0 || args.concurrency > 64 {
        return Err("concurrency must be between 1 and 64".into());
    }
    let plans = parse_plans(&read_bounded(&args.input)?)?;
    if plans.len() > MAX_BATCH_SIZE {
        return Err("batch exceeds 256 plans".into());
    }
    if plans.len() == 1 && !args.ndjson {
        let report = ProbeEngine::default()
            .execute(plans.into_iter().next().expect("one plan"))
            .await;
        return Ok((exit_code(&report) as i32, render(&report, true)));
    }
    let mut output = String::new();
    let mut code = 0;
    for (index, plan) in plans.into_iter().enumerate() {
        if args.fail_fast && code != 0 {
            break;
        }
        let report = ProbeEngine::default().execute(plan).await;
        code = code.max(exit_code(&report) as i32);
        let mut value = serde_json::to_value(&report).map_err(|e| e.to_string())?;
        value["execution_id"] = serde_json::Value::String(format!("batch-{index}"));
        output.push_str(&serde_json::to_string(&value).map_err(|e| e.to_string())?);
        output.push('\n');
    }
    Ok((code, output))
}

async fn run_compare(args: CompareArgs) -> Result<(i32, String), String> {
    let direct: ProbePlan =
        serde_json::from_str(&read_bounded(&args.direct)?).map_err(|e| e.to_string())?;
    let routed: ProbePlan =
        serde_json::from_str(&read_bounded(&args.routed)?).map_err(|e| e.to_string())?;
    if args.repeat == 0 || args.repeat > 100 {
        return Err("repeat must be between 1 and 100".into());
    }
    let mut reports = Vec::new();
    for _ in 0..args.repeat {
        reports.push(ProbeEngine::default().execute(direct.clone()).await);
        reports.push(ProbeEngine::default().execute(routed.clone()).await);
    }
    let timings: Vec<u64> = reports
        .iter()
        .flat_map(|report| report.probes.iter())
        .filter_map(|probe| probe.timing.as_ref().map(|timing| timing.total.as_micros()))
        .collect();
    let output = serde_json::json!({"kind":"comparison","repetitions":args.repeat,"connection_policy":"cold","statistics":statistics(&timings),"attempts":reports});
    Ok((
        0,
        if args.json {
            serde_json::to_string_pretty(&output).map_err(|e| e.to_string())?
        } else {
            format!(
                "comparison repetitions: {}\nattempts: {}\n",
                args.repeat,
                args.repeat * 2
            )
        },
    ))
}

fn parse_plans(text: &str) -> Result<Vec<ProbePlan>, String> {
    if let Ok(plan) = serde_json::from_str::<ProbePlan>(text) {
        return Ok(vec![plan]);
    }
    if let Ok(plans) = serde_json::from_str::<Vec<ProbePlan>>(text) {
        return Ok(plans);
    }
    Ok(serde_json::from_str::<Batch>(text)
        .map_err(|e| format!("invalid plan or batch: {e}"))?
        .plans)
}
#[derive(serde::Deserialize)]
struct Batch {
    plans: Vec<ProbePlan>,
}
fn read_bounded(path: &PathBuf) -> Result<String, String> {
    let reader: Box<dyn Read> = if path == &PathBuf::from("-") {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open(path).map_err(|e| e.to_string())?)
    };
    let mut bytes = Vec::new();
    reader
        .take((MAX_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_INPUT_BYTES {
        return Err("input exceeds 4 MiB".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn route(via: Option<&str>) -> Result<RouteSpec, String> {
    match via {
        Some(expression) if expression.trim().is_empty() => {
            Err("route expression must not be empty".into())
        }
        Some(expression) => Ok(RouteSpec::Eggress(eggprobe_core::EggressRoute {
            expression: expression.into(),
        })),
        None => Ok(RouteSpec::Direct),
    }
}
fn target(host: &str, port: Option<u16>) -> Result<TargetSpec, String> {
    TargetSpec::new(host, port).map_err(|e| e.to_string())
}
fn execution(timeout_ms: u64) -> ExecutionPolicy {
    ExecutionPolicy {
        deadline: eggprobe_core::DurationMicros::from_micros(timeout_ms.saturating_mul(1000)),
        repetitions: 1,
        retries: 0,
    }
}
fn base(
    target: TargetSpec,
    route: RouteSpec,
    probes: Vec<ProbeSpec>,
    timeout_ms: u64,
    assertions: Vec<AssertionSpec>,
) -> ProbePlan {
    ProbePlan {
        schema_version: SchemaVersion::INITIAL,
        target,
        route,
        probes,
        execution: execution(timeout_ms),
        assertions,
    }
}
fn plan_for_dns(args: &PrimitiveArgs) -> Result<ProbePlan, String> {
    Ok(base(
        target(&args.target, None)?,
        route(args.via.as_deref())?,
        vec![ProbeSpec::Dns],
        args.timeout_ms,
        vec![],
    ))
}
fn plan_for_tcp(args: &PrimitiveArgs) -> Result<ProbePlan, String> {
    let port = args.port.ok_or("--port is required")?;
    Ok(base(
        target(&args.target, Some(port))?,
        route(args.via.as_deref())?,
        vec![ProbeSpec::Tcp { port }],
        args.timeout_ms,
        vec![],
    ))
}
fn plan_for_tls(args: &TlsArgs) -> Result<ProbePlan, String> {
    let port = args.primitive.port.ok_or("--port is required")?;
    Ok(base(
        target(&args.primitive.target, Some(port))?,
        route(args.primitive.via.as_deref())?,
        vec![ProbeSpec::Tls {
            port,
            server_name: args.server_name.clone(),
        }],
        args.primitive.timeout_ms,
        vec![],
    ))
}
fn plan_for_http(args: &HttpArgs) -> Result<ProbePlan, String> {
    let host = parse_url_host(&args.url)?;
    Ok(base(
        target(&host, None)?,
        route(args.via.as_deref())?,
        vec![ProbeSpec::Http {
            url: args.url.clone(),
            method: args.method.clone(),
        }],
        args.timeout_ms,
        vec![],
    ))
}
fn plan_for_proxy(args: &ProxyArgs) -> Result<ProbePlan, String> {
    Ok(base(
        target(&args.target, Some(args.port))?,
        route(Some(args.via.as_str()))?,
        vec![ProbeSpec::Tcp { port: args.port }],
        args.timeout_ms,
        vec![],
    ))
}
fn plan_for_check(args: &CheckArgs) -> Result<ProbePlan, String> {
    let mut probes = vec![ProbeSpec::Dns, ProbeSpec::Tcp { port: args.port }];
    if let Some(url) = &args.url {
        probes.push(ProbeSpec::Http {
            url: url.clone(),
            method: "GET".into(),
        });
    }
    Ok(base(
        target(&args.target, Some(args.port))?,
        route(args.via.as_deref())?,
        probes,
        args.timeout_ms,
        vec![AssertionSpec {
            id: "http-status".into(),
            assertion: AssertionKind::HttpStatusRange {
                min: args.expect_status_min,
                max: args.expect_status_max,
            },
        }],
    ))
}
fn parse_url_host(url: &str) -> Result<String, String> {
    let start = url.find("://").ok_or("URL must be absolute")? + 3;
    let authority = url[start..]
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default();
    let authority = authority
        .rsplit_once('@')
        .map_or(authority, |(_, value)| value);
    let host = authority
        .strip_prefix('[')
        .and_then(|v| v.split_once(']').map(|(h, _)| h))
        .or_else(|| authority.rsplit_once(':').map(|(h, _)| h))
        .unwrap_or(authority);
    if host.is_empty() {
        return Err("URL host is empty".into());
    }
    Ok(host.into())
}
fn render(report: &ProbeReport, json: bool) -> String {
    if json {
        serde_json::to_string_pretty(report).expect("report serializes")
    } else {
        eggprobe_core::render::render_human(report)
    }
}

fn statistics(values: &[u64]) -> serde_json::Value {
    if values.is_empty() {
        return serde_json::json!({"count": 0, "successes": 0, "failures": 0, "unsupported": 0});
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let percentile =
        |percent: usize| sorted[(percent * (sorted.len() - 1) / 100).min(sorted.len() - 1)];
    serde_json::json!({"count": sorted.len(), "min": sorted[0], "max": sorted[sorted.len() - 1], "median": percentile(50), "p50": percentile(50), "p95": percentile(95)})
}
/// Render a report through the core renderer.
#[must_use]
pub fn render_report(report: &ProbeReport) -> String {
    eggprobe_core::render::render_human(report)
}
