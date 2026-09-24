# Domain Contract — `eggprobe-core::domain`

## 1. Purpose and Ownership

* **Owner:** `eggprobe-core` owns the canonical, JSON-first data contract shared by probe execution and the CLI adapter.
  * `crates/eggprobe-core/src/lib.rs:1-4`: "Canonical, JSON-first domain types… owns the data contract shared by future probe execution and the command-line adapter. It intentionally contains no network implementation."
  * `crates/eggprobe-core/src/lib.rs:6-8`: `#![deny(unsafe_code)]`, `#![warn(missing_docs)]`.
  * `crates/eggprobe-core/src/domain/mod.rs:1-11`: `domain` aggregates `error, finding, plan, probe, report, route, target, timing, version`.
  * Re-exports define the public contract surface: `crates/eggprobe-core/src/lib.rs:16-28`.
* **JSON-first means:**
  * Every domain type derives `serde::Serialize/Deserialize` + `schemars::JsonSchema`.
  * Input structs/enums use `#[serde(deny_unknown_fields)]` to reject forward-unknown fields.
  * Output uses `skip_serializing_if` to omit `None` / empty / zero values, keeping reports stable and minimal.
  * Durations have one canonical JSON unit: integer microseconds (`timing.rs`).
  * Versions are explicit strings, not integers (`version.rs`).
* **No I/O in domain:** validation is syntactic + cross-field only. No DNS resolution (`target.rs:39-44`), no sockets, no proxy execution. Semantic network policy (`TargetPolicy`) is exported from `engine`, not `domain` (`lib.rs:28`).

## 2. Domain Types

### 2.1 `domain/plan.rs` — Typed execution intent

**`ProbePlan` — `plan.rs:12-30`**

```rust
pub struct ProbePlan {
  pub schema_version: SchemaVersion,
  pub target: TargetSpec,
  pub route: RouteSpec, // may contain input-only credentials
  pub probes: Vec<ProbeSpec>, // ordered
  #[serde(default)] pub execution: ExecutionPolicy,
  #[serde(default)] pub assertions: Vec<AssertionSpec>,
}
```

* `execution` and `assertions` default when omitted, preserving backward-compatible JSON.

**`ProbeSpec` — `plan.rs:179-206`** — `#[serde(rename_all="snake_case", tag="kind")]`:
  * `dns` — no fields; resolve target name.
  * `tcp { port: u16 }` — direct or routed byte-stream.
  * `tls { port: u16, server_name?: String }` — standalone handshake.
  * `http { url: String, method: String }` — `method` defaults to `"GET"` via `default_http_method()` (`plan.rs:208-210`).

**`ExecutionPolicy` — `plan.rs:212-232`** — `{ deadline: DurationMicros, repetitions: u32, retries: u32 }`. Default: 30s / 1 / 0 (`plan.rs:224-232`).

**`AssertionSpec` / `AssertionKind` — `plan.rs:234-275`** — `{ id: String, assertion: AssertionKind }` with variants `http_status_range`, `required_http_version`, `required_alpn`, `required_tls_version`, `max_total_micros`.

**Validation — `ProbePlan::validate()` — `plan.rs:39-88`:**
1. `schema_version != CURRENT` → `UnsupportedSchema`; 2. zero deadline → `ZeroDeadline`; 3. zero repetitions → `ZeroRepetitions`; 4. `retries > 0` → `UnsupportedRetries`; 5. `HttpStatusRange min > max` → `InvalidAssertionRange`; 6. TCP/TLS `port 0` → `InvalidPort`; 7. HTTP URL authority checks (scheme `http/https`, no userinfo, host must equal `target.host` via `same_host()`, port must match when `target.port` set) else `InvalidHttpUrl` / `TargetMismatch` / `PortMismatch`.

### 2.2 `domain/target.rs` — Validated target input

**`TargetSpec` — `target.rs:9-18`** — `{ host: String, port?: u16 }`. Custom `Deserialize` via private `RawTarget` so deserialization always validates. Rules (`target.rs:58-89`): non-empty, ≤253 chars, no whitespace, `port != 0`, IP literals bypass label checks, else per-label (non-empty, ≤63, no leading/trailing `-`, alphanumeric/`-`). No resolution, no private-IP check here.

### 2.3 `domain/route.rs` — Input routes and redaction boundary

**`RouteSpec` — `route.rs:10-17`** — `direct` | `eggress(EggressRoute{expression})`. `summary()` (`route.rs:19-28`) drops the expression: `Direct → RouteSummary::Direct`, `Eggress(_) → RouteSummary::Eggress`. Redacted `Debug`/`Display` print `<redacted>` (`route.rs:30-49`); `EggressRoute::Debug` likewise (`route.rs:59-66`).

### 2.4 `domain/report.rs` — Canonical report envelope

`ToolProvenance{name, version: ToolVersion}` (`report.rs:14-22`); `TargetSummary{host, port?}` with `From<&TargetSpec>` (`report.rs:24-42`); `RouteSummary::{Direct, Eggress}` with no data fields (`report.rs:44-52`); `ReportStatus::{Ok, Failed, Unsupported, Cancelled}` (`report.rs:54-66`); `ProbeReport{schema_version, tool, execution_id, target, route, status, probes, findings, warnings}` with `from_plan()` (`report.rs:94-115`) and `to_json()` pretty serializer (`report.rs:117-125`).

### 2.5 `domain/probe.rs` — Result envelopes and evidence

`ProbeKind{dns,tcp,tls,http}` (`probe.rs:8-20`); `ProbeStatus{ok,failed,unsupported,cancelled}` (`probe.rs:22-34`); `ProbeEvidence` adjacently tagged `kind`+`data` (`probe.rs:36-48`); `DnsEvidence{addresses, resolution_scope: Client}` (local-resolver semantics documented, `probe.rs:50-67`); `TcpEvidence{peer?, local?, attempts}` (`probe.rs:69-82`); `TlsEvidence{version?, alpn?, cipher_suite?}` (`probe.rs:84-97`); `HttpEvidence{status?, protocol?, body_sample_bytes}` (`probe.rs:99-112`); `ProbeResult{kind, status, timing?, error?, evidence?, unavailable[], findings[]}` (`probe.rs:119-142`).

### 2.6 `domain/finding.rs` — Assertion findings

`FindingSeverity{info,warning,error}` (`finding.rs:6-16`); `FindingOutcome{passed,failed,unavailable}` (`finding.rs:18-28`); `Finding{assertion_id, severity, outcome, message}` (`finding.rs:30-42`) — "normalized assertion result, not a replacement for probe status/errors."

### 2.7 `domain/timing.rs` — Canonical timing unit

`DurationMicros(u64)` transparent, JSON bare integer micros, `from_secs` saturating (`timing.rs:9-32`); `Timing{total, phases[]}` — only truthfully observed phases (`timing.rs:35-44`); `PhaseTiming{stage: DiagnosticStage, duration}` (`timing.rs:46-54`).

### 2.8 `domain/version.rs` — Schema and tool versions

`SchemaVersion{major, minor}` (`version.rs:11-16`): JSON `"major.minor"`, schema pattern `^[0-9]+\.[0-9]+$`, `INITIAL = 0.1`, `CURRENT = 0.3`. `ToolVersion(String)` transparent, rejects empty (`version.rs:85-126`). `VersionParseError{InvalidFormat, InvalidNumber, EmptyToolVersion}`.

### 2.9 `domain/error.rs` — Structured diagnostics

`DiagnosticErrorKind`: 13 variants `dns, timeout, connection_refused, network_unreachable, host_unreachable, authentication, tls, protocol, policy, io, unsupported, internal, other` (`error.rs:7-36`). `DiagnosticStage`: 10 variants `resolution, direct_connect, hop_connect, hop_handshake, tls_handshake, request, response_headers, body, deadline, other` (`error.rs:38-62`). `DiagnosticError{kind, stage, message (bounded, credential-stripped), attempt?, route_hop_index?, route_protocol?}` (`error.rs:64-83`).

## 3. Relationships

```text
ProbePlan (plan.rs:15)
├─ schema_version — must equal CURRENT (0.3)
├─ target: TargetSpec ──► TargetSummary via From<&TargetSpec>
├─ route: RouteSpec ──► RouteSummary via summary() — redaction boundary
├─ probes: Vec<ProbeSpec> — Dns|Tcp|Tls|Http, ordered
├─ execution: ExecutionPolicy — deadline/repetitions/retries
└─ assertions: Vec<AssertionSpec> — deferred, typed

ProbeReport (report.rs:71) — built by from_plan()
├─ schema_version ← plan.schema_version (no upgrade; equality-gated)
├─ tool: ToolProvenance — producer, not contract
├─ target: TargetSummary — safe clone of plan target
├─ route: RouteSummary — Direct|Eggress only, no expression
├─ status: ReportStatus — aggregate; children keep own ProbeStatus
├─ probes: Vec<ProbeResult> — ordered, 1:1 with intent
└─ findings / warnings — assertion results / non-fatal notes
```

Findings never substitute `ProbeStatus`/`DiagnosticError`. `DiagnosticStage` links failure location to observed phases. `MaxTotalMicros` assertions evaluate `Timing.total`.

## 4. Invariants

1. Core owns contract, no network (`lib.rs:1-4`).
2. Strict JSON: `deny_unknown_fields` on all input/output structs.
3. Version gating: any `schema_version != CURRENT (0.3)` rejected.
4. Redaction: `RouteSpec` may hold credentials; `ProbeReport` holds only `RouteSummary`; `Debug`/`Display` emit `<redacted>`; `DiagnosticError.message` credential-stripped.
5. Target authority: HTTP URLs must match `TargetSpec.host` and (when set) port; scheme limited to `http|https`, no userinfo.
6. Syntactic-only target validation; private-IP policy lives in `engine::TargetPolicy`, not domain.
7. Execution bounds: non-zero deadline/repetitions, `retries == 0`, micros as sole unit.
8. Assertion soundness: `min <= max`, stable `id` → `Finding.assertion_id`, `Unavailable` for missing evidence.
9. Failure vs finding separation: `ProbeStatus` + `DiagnosticError` report what happened; `Finding` reports whether expectations held.
10. Truthful observability: `phases` only observed, zero-valued fields omitted, DNS scope `Client` documents local-resolver semantics, `unavailable[]` marks seam gaps.

## 5. Key Code References

Ownership/no-network: `crates/eggprobe-core/src/lib.rs:1-4`; contract surface: `lib.rs:16-28`; module index: `domain/mod.rs:1-11`; `ProbePlan` + `validate`: `domain/plan.rs:12-88`; `ProbeSpec`/`ExecutionPolicy`/`AssertionSpec`: `domain/plan.rs:179-275`; `TargetSpec`: `domain/target.rs:9-89`; `RouteSpec` + redaction: `domain/route.rs:10-66`; report envelope: `domain/report.rs:14-126`; probe evidence: `domain/probe.rs:8-142`; findings: `domain/finding.rs:6-42`; timing: `domain/timing.rs:8-54`; versions: `domain/version.rs:11-140`; errors: `domain/error.rs:6-83`.

## 6. Review Checklist / Risks

* Unknown-field strictness on every new field; missing attribute silently widens the contract.
* Version bump discipline: equality-gated `CURRENT`; additive change needs minor-bump policy + rejection tests.
* Redaction leaks: verify no `expression` reaches reports, logs, `Debug`/`Display`, or error messages; `summary()` must stay the sole conversion path.
* Target matching edge cases: trailing-dot/case/IP normalization, IPv6 brackets, IDN, default-port inference, `target.port=None` wildcard, triple `port 0` layers.
* Semantic target policy gap: strict vs `AllowPrivate` enforced in engine on resolved IPs, not just literal hosts.
* Retries currently hard-rejected; adding attempts needs per-attempt contract and timing aggregation rules.
* Assertion ↔ evidence coverage: each `AssertionKind` needs defined `Unavailable` (not `Passed`) behavior for missing evidence.
* Error taxonomy misuse: audit `Other`/`Internal`/`Unsupported` overuse; `message` bounded and credential-free.
* Timing truthfulness: `total` vs `phases`, single micros unit, omitted-zero fields.
* Report aggregation: `ReportStatus` vs per-probe `ProbeStatus` vs `Finding.outcome` can diverge — pin precedence used by exit-code logic.
* DNS scope: only `Client` exists; new proxy-resolved answers need new scope variants.
* Serde/schema drift: tags, envelopes, transparent wrappers, and `SchemaVersion` regex must round-trip through schema generation.
