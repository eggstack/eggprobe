# Eggprobe Architecture — Bird's-Eye View

Eggprobe is a pre-release, JSON-first network diagnostics project.
The canonical plan/report contract lives in `eggprobe-core`; the `eggprobe`
binary (`eggprobe-cli`) is a thin presentation adapter and does not own networking.

- Workspace: `Cargo.toml` — members `crates/eggprobe-core`, `crates/eggprobe-cli`, resolver 3, edition 2021, MSRV 1.89, `unsafe_code = forbid`.
- Active contract: schema 0.3 (`schemas/plan-0.3.json`, `schemas/report-0.3.json`). `0.1`/`0.2` are retained historical evidence; the binary rejects earlier plan versions.
- Founding rule: `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md` — machine JSON is the contract; human rendering is downstream.
- Release rule: `plans/adrs/ADR-0002-release-producer-consumer-ownership.md` — qualified archives + checksums + manual install; Eggpack owns producer construction, Eggup owns consumer update. First archive release is not blocked on either integration.
- Operator entry point: `docs/operator.md`. Planning control surface: `plans/registry.md`.

## Module map

| # | Module / capability | Owner | Deep dive |
|---|---|---|---|
| 1 | Domain contract — `Plan`, `Target`, `ProbeSpec`, `Report`, `Finding`, `Route`, `Timing`, `Version`, `Error` | `eggprobe-core::domain` | [domain-contract.md](domain-contract.md) |
| 2 | Probe engine / transport — direct DNS/TCP/TLS, Eggfetch HTTP, Eggress routing, deadlines | `eggprobe-core::engine` | [engine-transport.md](engine-transport.md) |
| 3 | CLI / automation — `dns/tcp/tls/http/proxy/check/run/compare`, batch NDJSON, fail-fast, concurrency | `eggprobe-cli` | [cli-automation.md](cli-automation.md) |
| 4 | Assertions / findings / exit codes — typed expectations vs observations, `DiagnosticError` taxonomy | `eggprobe-core::assertions` + `domain::{finding,error,report}` | [assertions-findings.md](assertions-findings.md) |
| 5 | Schema contract / versioning — schemars generation, `SchemaVersion` gating, additive-only evolution | `eggprobe-core::schema` + `schemas/` | [schema-contract.md](schema-contract.md) |
| 6 | Rendering / privacy — pretty JSON vs NDJSON vs human, `RouteSpec` → `RouteSummary` redaction, resolver scope | `eggprobe-core::render` + `domain::route` | [render-privacy.md](render-privacy.md) |
| 7 | Release / operations — archives, qualification matrix, MSRV, verification, operator surface | repo + `.github/` + `docs/operator.md` | [release-operations.md](release-operations.md) |

## Tools

- **Probes:** direct DNS (system resolver, `client` scope), direct TCP (serial Happy-Eyeballs-style address trial), direct TLS (rustls + webpki-roots, ALPN `h2,http/1.1`), Eggfetch-backed HTTP (H1/H2, custom `PolicyDialer`/`RouteDialer`, bounded 4 KiB body sample, per-phase timings explicitly unavailable).
- **Routing:** listener-free Eggress client (`eggress-core`/`eggress-embed` 1.0.8, `pproxy-compat`), TCP byte-stream dialer only (no QUIC/H3), no env-proxy fallback, typed hop errors with `route_hop_index`/`route_protocol` provenance and fixed redacted messages.
- **Assertions:** `http_status_range`, `required_http_version`, `required_alpn`, `required_tls_version`, `max_total_micros`; evaluated purely into `Finding{passed,failed,unavailable}`; HTTP 4xx/5xx stay `Ok` probes until an assertion fails them.
- **Automation:** `run` (plan-file / `Vec<Plan>` / `{plans:[...]}`, stdin `-`, 4 MiB / 256-plan bounds, `JoinSet` concurrency 1–64 default 4, input-order NDJSON with `batch-N` execution IDs, `--fail-fast`), `compare` (direct-vs-routed, `repeat` 1–100, cold policy, nearest-rank `ceil(p*N/100)` percentiles, median delta).
- **Presentation:** pretty JSON (single), compact NDJSON (batch), short human (fallback); stdout is machine data only under `--json`/`--ndjson`, diagnostics go to stderr; exit codes `0/1/2/3/130`.
- **Schemas:** `cargo run -p eggprobe-core --example generate-schemas --locked` regenerates `schemas/*-0.3.json` (with pinned `const "0.3"`); `deny_unknown_fields` everywhere; `DurationMicros` integer micros as the sole duration unit.
- **Verification:** `cargo fmt --check`, `cargo check`, `cargo clippy -- -D warnings`, `cargo test`, `cargo +1.89.0 check`, `cargo tree`, `cargo audit`; CI quality matrix (ubuntu/macos/windows) + MSRV + audit jobs; release packaging matrix (linux x86_64/aarch64, macos x86_64/arm64, windows x86_64).

## How everything fits together

```text
                    schemas/*.json (0.3 active)
                              ^
                              | generated / verified
                              |
ProbePlan (domain) --> ProbeEngine (core) --> ProbeReport (domain)
     ^                       |                         |
     |                       | Eggfetch HTTP           +-- evaluate_assertions --> findings
     |                       | Eggress routes          +-- exit_code --> 0/1/130
     |                       |
CLI plan builders            +-- render (JSON/human)
(dns/tcp/tls/http/          +-- stdout (machine) / stderr (logs)
 proxy/check/run/compare)
```

1. CLI parses args (`eggprobe-cli`) and builds a `ProbePlan` with `schema_version: CURRENT (0.3)`.
2. `ProbePlan::validate()` gates version, deadline, ports, HTTP authority match, assertion ranges.
3. `ProbeEngine::execute` runs probes sequentially under a finite outer deadline, mapping failures to `DiagnosticError{kind,stage}` and successes to typed `ProbeEvidence`.
4. `evaluate_assertions` maps `AssertionSpec`s to report-level `Finding`s without mutating probe evidence.
5. Rendering selects pretty JSON / NDJSON / human; route credentials never cross the `RouteSpec` → `RouteSummary` boundary (`{"kind":"eggress"}`, `eggress(<redacted>)`).
6. Process exit folds `exit_code` / `CliError::code` / `combine_exit_codes` into `0/1/2/3/130`.
7. Releases ship the `eggprobe` binary as qualified archives with checksums; operator docs describe install, quick-start, troubleshooting, and compare semantics.

## Review workflow

Each deep-dive file is a self-contained review unit with purpose, key types/functions, invariants, code references (`file:line`), and a reviewer checklist. Start here, then follow the index table above in dependency order (contract → engine → CLI → assertions → schema → render → release).
