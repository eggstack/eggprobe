# Assertions, Findings, Exit Codes

> Evaluate typed assertions without changing the underlying observations. — `crates/eggprobe-core/src/assertions.rs:8`

Evidence is observation (`ProbeStatus` + `ProbeEvidence` + `DiagnosticError`); findings are expectations (`ProbeReport.findings`). Evaluation never mutates probes: `evaluate_assertions(report, assertions) -> Vec<Finding>` (`assertions.rs:10-15`); CLI assigns to `report.findings` after execution (`eggprobe-cli/src/lib.rs:230-231,245-246,291`).

HTTP 4xx/5xx are observations, not failures: engine returns `Ok(HttpEvidence{status})` for any received status (`engine.rs:353-370`); only `HttpStatusRange` turns it into `FindingOutcome::Failed` (`assertions.rs:19-33`).

## Declaration and evaluation

`AssertionSpec{id, assertion: AssertionKind}` (`domain/plan.rs:234-275`), `deny_unknown_fields`, five kinds: `http_status_range{min,max}`, `required_http_version{version}`, `required_alpn{value}`, `required_tls_version{version}`, `max_total_micros{micros}`. Only `min <= max` validated (`plan.rs:54-63`); sole CLI producer is `plan_for_check` (`http-status` when `--url` present).

`evaluate_one` (`assertions.rs:17-96`) always wraps as `Finding{assertion_id, severity: Error, outcome, message}`:

| Kind | Lookup | Passed / Failed / Unavailable |
|---|---|---|
| `HttpStatusRange` (`:19-33`) | first `ProbeEvidence::Http` (`:98-106`) | status in range / status outside / `status None`, non-Ok probe, or no HTTP evidence |
| `RequiredHttpVersion` (`:34-47`) | same | `protocol == version` (exact, e.g. `HTTP/2.0`) / mismatch / `None` or missing evidence |
| `RequiredAlpn` / `RequiredTlsVersion` (`:48-69`) | first `ProbeEvidence::Tls` (`:108-116`) | exact `alpn`/`version` match (e.g. `h2`, `TLSv1_3`) / mismatch / `None` or missing |
| `MaxTotalMicros` (`:70-88`) | first `probe.timing` found | `total <= micros` / exceeds / no timing |

`Finding{assertion_id, severity, outcome, message}` (`domain/finding.rs:30-42`): severity `info/warning/error`, outcome `passed/failed/unavailable`; "not a replacement for probe status/errors." Evaluator always emits `Error` (`assertions.rs:92`) — `Info`/`Warning` schema-reserved but dead. Findings live only at report level (`domain/report.rs:86-88`); `ProbeResult.findings` exists but engine zeroes it.

## `exit_code()`, `ReportStatus`, CLI wiring

`ExitCode{Success=0, Negative=1, Invalid=2, Internal=3, Interrupted=130}`; `exit_code()` (`assertions.rs:139-156`): `Cancelled→130`; `Failed|Unsupported→1`; any finding `!= Passed` (including `Unavailable`)→1; else 0. `Invalid`/`Internal` never come from `exit_code()` — only `CliError::code()`.

`ReportStatus{Ok,Failed,Unsupported,Cancelled}` (`domain/report.rs:54-66`); engine fold `Failed > Unsupported > Cancelled > Ok` (`engine.rs:110-118`); `validate()` error and outer deadline both produce `Failed` + warning. `execute_probe` currently emits only `Ok`/`Failed`, so `Unsupported`/`Cancelled` arms are defensive.

CLI (`eggprobe-cli/src/lib.rs`): `run_single` + single `run` evaluate then `exit_code`; batch validates upfront, per-task evaluates, folds `combine_exit_codes` (`130>3>2>1>0`, `lib.rs:601-609`), `fail_fast` stops spawning once non-zero; `run_compare` does **not** call `evaluate_assertions` — exit reflects `ReportStatus` only.

## `DiagnosticError` taxonomy (`domain/error.rs:1-83`)

Kind (13): `dns, timeout, connection_refused, network_unreachable, host_unreachable, authentication, tls, protocol, policy, io, unsupported, internal, other`. Stage (10): `resolution, direct_connect, hop_connect, hop_handshake, tls_handshake, request, response_headers, body, deadline, other`. `DiagnosticError{kind, stage, message (bounded, credential-stripped), attempt?, route_hop_index?, route_protocol?}`, `deny_unknown_fields`.

Producers (`engine.rs`): `resolve_addresses` (DNS/policy); direct TCP `normalize_io` (`ConnectionRefused/Timeout/…`, `kind().to_string()` only); routed TCP/TLS `diagnostic_from_route_error` (kind+stage mapped, fixed `routed …` messages, hop provenance); TLS SNI/handshake; HTTP `normalize_http_failure` (timeout → network kind → routed unwrap → `DialErrorKind` → string-kind fallback); outer deadline (`Timeout/Deadline` per probe); `PolicyDialer` (`Rejected`/`Other` for eggfetch).

## Key Code References

Evaluation: `assertions.rs:10-116,122-156`; specs: `domain/plan.rs:234-275,54-63`; findings: `domain/finding.rs:9-42`; reports: `domain/report.rs:57-115`; evidence: `domain/probe.rs:11-142`; errors: `domain/error.rs:9-83`; engine reductions/mappings: `engine.rs:69-74,85-94,110-118,714-747,832-871,881-1093,1000-1046`; CLI: `eggprobe-cli/src/lib.rs:220-341,478-507,601-609,150-157`.

## Review Checklist / Risks

* `Unavailable` is negative (fails CI) — confirm intended vs warning.
* Severity constant `Error` — add per-spec severity or remove `Info`/`Warning`.
* First-match evidence (`find_map`) arbitrary under repetitions/mixed probes — document or reduce (all/any/last).
* Brittle exact string equality (`HTTP/2.0`, `TLSv1_3`, `h2`) — normalize or document canonical values.
* Inconsistent failed-probe messaging across arms — unify deliberately.
* Thin coverage: no DNS/TCP assertions; cipher/peer/attempts/body unassertable; `MaxTotalMicros` family-agnostic.
* Validation gap: empty `version`/`value`, `micros: 0`, empty/duplicate `id`s pass — decide fail-fast vs `Unavailable`.
* `Unsupported` collapses to `1` — automation can't distinguish "unsupported" from "failed".
* `run_compare` ignores assertions silently.
* Dead `ProbeResult.findings` — populate or remove to avoid dual-source confusion.
* Erased diagnostics limit debuggability — ensure `warnings`/`unavailable[]` carry signal without leaking credentials.
* Redaction: new error mappings must use fixed strings, never dependency `to_string()` with URLs/credentials.
