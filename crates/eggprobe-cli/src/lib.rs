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
use tokio::task::JoinSet;

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
    Route(PrimitiveArgs),
    Tcp(PrimitiveArgs),
    Tls(TlsArgs),
    Http(HttpArgs),
    Udp(UdpArgs),
    Trace(TraceArgs),
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
/// Direct UDP controls.
#[derive(Clone, Debug, Args)]
pub struct UdpArgs {
    #[command(flatten)]
    pub primitive: PrimitiveArgs,
    /// Optional request payload text, bounded to 1200 bytes.
    #[arg(long)]
    pub payload: Option<String>,
    /// Wait for one bounded reply before completing.
    #[arg(long)]
    pub receive: bool,
}
/// Direct traceroute controls. No port applies: the backend traces the
/// target address with TTL-limited UDP probes.
#[derive(Clone, Debug, Args)]
pub struct TraceArgs {
    pub target: String,
    #[arg(long)]
    pub via: Option<String>,
    #[arg(long, default_value_t = 30)]
    pub max_hops: u8,
    #[arg(long, default_value_t = 3)]
    pub attempts: u8,
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

/// A classified CLI failure. Invalid input is distinct from an internal
/// execution or serialization invariant failure.
#[derive(Debug)]
pub enum CliError {
    /// Invalid invocation, input, or plan.
    Invalid(String),
    /// Internal execution, task, or serialization failure.
    Internal(String),
}

impl CliError {
    fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Return the documented process code for this failure.
    #[must_use]
    pub const fn code(&self) -> i32 {
        match self {
            Self::Invalid(_) => 2,
            Self::Internal(_) => 3,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(message) | Self::Internal(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for CliError {}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}

impl From<&str> for CliError {
    fn from(message: &str) -> Self {
        Self::Invalid(message.into())
    }
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
pub async fn execute(cli: Cli) -> Result<(i32, String), CliError> {
    let Some(command) = cli.command else {
        return Ok((0, "Run `eggprobe --help` for commands.\n".into()));
    };
    match command {
        Command::Dns(args) => run_single(plan_for_dns(&args)?, args.json, args.strict_target).await,
        Command::Route(args) => {
            run_single(plan_for_route(&args)?, args.json, args.strict_target).await
        }
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
        Command::Udp(args) => {
            run_single(
                plan_for_udp(&args)?,
                args.primitive.json,
                args.primitive.strict_target,
            )
            .await
        }
        Command::Trace(args) => {
            run_single(plan_for_trace(&args)?, args.json, args.strict_target).await
        }
        Command::Proxy(args) => run_single(plan_for_proxy(&args)?, args.json, false).await,
        Command::Check(args) => {
            run_single(plan_for_check(&args)?, args.json, args.strict_target).await
        }
        Command::Run(args) => run_input(args).await,
        Command::Compare(args) => run_compare(args).await,
    }
}

async fn run_single(plan: ProbePlan, json: bool, strict: bool) -> Result<(i32, String), CliError> {
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

async fn run_input(args: RunArgs) -> Result<(i32, String), CliError> {
    if args.concurrency == 0 || args.concurrency > 64 {
        return Err("concurrency must be between 1 and 64".into());
    }
    let plans = parse_plans(&read_bounded(&args.input)?)?;
    if plans.len() > MAX_BATCH_SIZE {
        return Err("batch exceeds 256 plans".into());
    }
    if plans.len() == 1 && !args.ndjson {
        let plan = plans.into_iter().next().expect("one plan");
        let mut report = ProbeEngine::default().execute(plan.clone()).await;
        report.findings = evaluate_assertions(&report, &plan.assertions);
        return Ok((exit_code(&report) as i32, render(&report, true)));
    }
    let indexed = plans.into_iter().enumerate().collect::<Vec<_>>();
    for (_, plan) in &indexed {
        plan.validate().map_err(|error| error.to_string())?;
    }
    let mut reports = vec![None; indexed.len()];
    let mut tasks = JoinSet::new();
    let mut next = 0;
    let mut code = 0;
    while next < indexed.len() && tasks.len() < args.concurrency {
        spawn_batch_task(&mut tasks, indexed[next].clone());
        next += 1;
    }
    while let Some(result) = tasks.join_next().await {
        let (index, report) = result.map_err(|error| CliError::internal(error.to_string()))?;
        code = combine_exit_codes(code, exit_code(&report) as i32);
        reports[index] = Some(report);
        if !(args.fail_fast && code != 0) && next < indexed.len() {
            spawn_batch_task(&mut tasks, indexed[next].clone());
            next += 1;
        }
    }
    let mut output = String::new();
    for (index, report) in reports.into_iter().enumerate() {
        let Some(report) = report else {
            continue;
        };
        let mut value = serde_json::to_value(&report).map_err(|error| {
            CliError::internal(format!("failed to serialize batch result {index}: {error}"))
        })?;
        value["execution_id"] = serde_json::Value::String(format!("batch-{index}"));
        output.push_str(&serde_json::to_string(&value).map_err(|error| {
            CliError::internal(format!("failed to serialize batch result {index}: {error}"))
        })?);
        output.push('\n');
    }
    Ok((code, output))
}

fn spawn_batch_task(tasks: &mut JoinSet<(usize, ProbeReport)>, item: (usize, ProbePlan)) {
    tasks.spawn(async move {
        let (index, plan) = item;
        let mut report = ProbeEngine::default().execute(plan.clone()).await;
        report.findings = evaluate_assertions(&report, &plan.assertions);
        (index, report)
    });
}

async fn run_compare(args: CompareArgs) -> Result<(i32, String), CliError> {
    let direct: ProbePlan =
        serde_json::from_str(&read_bounded(&args.direct)?).map_err(|e| e.to_string())?;
    let routed: ProbePlan =
        serde_json::from_str(&read_bounded(&args.routed)?).map_err(|e| e.to_string())?;
    if args.repeat == 0 || args.repeat > 100 {
        return Err("repeat must be between 1 and 100".into());
    }
    direct.validate().map_err(|error| error.to_string())?;
    routed.validate().map_err(|error| error.to_string())?;
    validate_comparison(&direct, &routed)?;
    let mut direct_reports = Vec::with_capacity(args.repeat as usize);
    let mut routed_reports = Vec::with_capacity(args.repeat as usize);
    for _ in 0..args.repeat {
        direct_reports.push(ProbeEngine::default().execute(direct.clone()).await);
        routed_reports.push(ProbeEngine::default().execute(routed.clone()).await);
    }
    let direct_stats = statistics(&direct_reports);
    let routed_stats = statistics(&routed_reports);
    let delta = comparable_delta(&direct_stats, &routed_stats);
    let code = direct_reports
        .iter()
        .chain(routed_reports.iter())
        .map(exit_code)
        .fold(0, |current, next| combine_exit_codes(current, next as i32));
    let output = serde_json::json!({
        "kind": "comparison",
        "repetitions": args.repeat,
        "connection_policy": "cold",
        "direct": direct_stats,
        "routed": routed_stats,
        "delta": delta,
        "attempts": {"direct": &direct_reports, "routed": &routed_reports}
    });
    Ok((
        code,
        if args.json {
            serde_json::to_string_pretty(&output).map_err(|e| CliError::internal(e.to_string()))?
        } else {
            format!(
                "comparison repetitions: {}\ndirect: {}\nrouted: {}\n",
                args.repeat, direct_stats["samples"], routed_stats["samples"]
            )
        },
    ))
}

fn validate_comparison(direct: &ProbePlan, routed: &ProbePlan) -> Result<(), CliError> {
    if !matches!(direct.route, RouteSpec::Direct) {
        return Err("compare direct input must use the direct route".into());
    }
    if !matches!(routed.route, RouteSpec::Eggress(_)) {
        return Err("compare routed input must use an Eggress route".into());
    }
    if direct.target != routed.target || direct.probes != routed.probes {
        return Err("compare inputs must have the same target and probe families".into());
    }
    Ok(())
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
        schema_version: SchemaVersion::CURRENT,
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
fn plan_for_route(args: &PrimitiveArgs) -> Result<ProbePlan, String> {
    Ok(base(
        target(&args.target, args.port)?,
        route(args.via.as_deref())?,
        vec![ProbeSpec::Route],
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
fn plan_for_udp(args: &UdpArgs) -> Result<ProbePlan, String> {
    let port = args.primitive.port.ok_or("--port is required")?;
    let payload = args.payload.clone().unwrap_or_default().into_bytes();
    if payload.len() > 1200 {
        return Err("UDP payload exceeds 1200 bytes".into());
    }
    Ok(base(
        target(&args.primitive.target, Some(port))?,
        route(args.primitive.via.as_deref())?,
        vec![ProbeSpec::Udp {
            port,
            payload,
            receive: args.receive,
        }],
        args.primitive.timeout_ms,
        vec![],
    ))
}
fn plan_for_trace(args: &TraceArgs) -> Result<ProbePlan, String> {
    Ok(base(
        target(&args.target, None)?,
        route(args.via.as_deref())?,
        vec![ProbeSpec::Trace {
            max_hops: args.max_hops,
            attempts_per_hop: args.attempts,
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
    if args.expect_status_min > args.expect_status_max {
        return Err(format!(
            "HTTP status assertion minimum {} exceeds maximum {}",
            args.expect_status_min, args.expect_status_max
        ));
    }
    let mut probes = vec![ProbeSpec::Dns, ProbeSpec::Tcp { port: args.port }];
    let mut assertions = Vec::new();
    if let Some(url) = &args.url {
        probes.push(ProbeSpec::Http {
            url: url.clone(),
            method: "GET".into(),
        });
        assertions.push(AssertionSpec {
            id: "http-status".into(),
            assertion: AssertionKind::HttpStatusRange {
                min: args.expect_status_min,
                max: args.expect_status_max,
            },
        });
    }
    Ok(base(
        target(&args.target, Some(args.port))?,
        route(args.via.as_deref())?,
        probes,
        args.timeout_ms,
        assertions,
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

fn statistics(reports: &[ProbeReport]) -> serde_json::Value {
    let samples: Vec<u64> = reports
        .iter()
        .flat_map(|report| report.probes.iter())
        .filter(|probe| probe.status == eggprobe_core::ProbeStatus::Ok)
        .filter_map(|probe| probe.timing.as_ref().map(|timing| timing.total.as_micros()))
        .collect();
    let successes = reports
        .iter()
        .filter(|report| report.status == eggprobe_core::ReportStatus::Ok)
        .count();
    let failures = reports
        .iter()
        .filter(|report| report.status == eggprobe_core::ReportStatus::Failed)
        .count();
    let unsupported = reports
        .iter()
        .filter(|report| report.status == eggprobe_core::ReportStatus::Unsupported)
        .count();
    if samples.is_empty() {
        return serde_json::json!({
            "attempts": reports.len(),
            "samples": 0,
            "successes": successes,
            "failures": failures,
            "unsupported": unsupported
        });
    }
    let mut sorted = samples;
    sorted.sort_unstable();
    let percentile = |percent: usize| {
        let rank = (percent * sorted.len()).div_ceil(100).max(1) - 1;
        sorted[rank.min(sorted.len() - 1)]
    };
    serde_json::json!({
        "attempts": reports.len(),
        "samples": sorted.len(),
        "successes": successes,
        "failures": failures,
        "unsupported": unsupported,
        "min": sorted[0],
        "max": sorted[sorted.len() - 1],
        "median": percentile(50),
        "p50": percentile(50),
        "p95": percentile(95)
    })
}

fn comparable_delta(direct: &serde_json::Value, routed: &serde_json::Value) -> serde_json::Value {
    match (
        direct.get("median").and_then(serde_json::Value::as_u64),
        routed.get("median").and_then(serde_json::Value::as_u64),
    ) {
        (Some(direct), Some(routed)) => serde_json::json!({
            "median_absolute_micros": i128::from(routed) - i128::from(direct),
            "median_relative": if direct == 0 { serde_json::Value::Null } else {
                serde_json::json!({
                    "numerator_micros": i128::from(routed) - i128::from(direct),
                    "denominator_micros": direct
                })
            }
        }),
        _ => serde_json::Value::Null,
    }
}

fn combine_exit_codes(current: i32, next: i32) -> i32 {
    match (current, next) {
        (130, _) | (_, 130) => 130,
        (3, _) | (_, 3) => 3,
        (2, _) | (_, 2) => 2,
        (1, _) | (_, 1) => 1,
        _ => 0,
    }
}
/// Render a report through the core renderer.
#[must_use]
pub fn render_report(report: &ProbeReport) -> String {
    eggprobe_core::render::render_human(report)
}
