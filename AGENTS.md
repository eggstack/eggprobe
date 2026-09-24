# AGENTS.md

## Ownership — do not invert
- `crates/eggprobe-core` owns the JSON-first contract + all networking: `domain` (Plan/Report, no I/O), `engine` (DNS/TCP/TLS direct, Eggfetch HTTP, Eggress routing), `assertions`, `render`, `schema`. It intentionally contains no CLI. → `architecture/overview.md`, `architecture/domain-contract.md`
- `crates/eggprobe-cli` (`eggprobe` bin, `src/lib.rs` + `src/main.rs`) is a thin Clap adapter: arg parsing, plan builders, `run`/`compare` orchestration, output selection, exit codes. It owns no networking. → `architecture/cli-automation.md`
- Rust types are authoritative; `schemas/plan-0.3.json` + `schemas/report-0.3.json` are generated artifacts. `*-0.1/0.2.json` are immutable historical evidence — never edit. → `architecture/schema-contract.md`, `schemas/README.md`
- No repo-local skills config (no `.skills/`, `opencode.json`, `.opencode/`); `architecture/` is the agent guide. Start at `architecture/overview.md`, then dependency order contract → engine → CLI. Operator surface: `docs/operator.md`. Planning control surface: `plans/registry.md` + `plans/adrs/ADR-0001*` (JSON-first) and `ADR-0002*` (release ownership).

## Toolchain / lints → `architecture/release-operations.md` §2
- MSRV 1.89.0 (`rust-toolchain.toml`, `rust-version="1.89"`, edition 2021, resolver 3). `unsafe_code = forbid`; clippy `all` + `pedantic` = warn, CI denies warnings.
- Always pass `--locked` (`Cargo.lock` is authoritative; `cargo tree --locked` / `cargo audit` are part of verification).

## Verification (exact order, from `README.md`)
```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```
- Focused: `cargo test -p eggprobe-core --all-features --locked [filter]`, `cargo test -p eggprobe-cli --all-features --locked [filter]`.
- CI (`ci.yml`): fmt+clippy+test matrix on ubuntu/macos/windows; separate MSRV `cargo check --workspace --all-targets --locked`; audit job uses an explicit `dtolnay/rust-toolchain@stable` + pinned `cargo-audit` install + `cargo +stable audit` so the audit tool does not inherit the product 1.89 MSRV. Canonical JSON fixtures and schemas are LF-locked via `.gitattributes` so Windows checkout cannot mutate the contract bytes.
- Regenerate schemas after any domain change, before merge: `cargo run -p eggprobe-core --example generate-schemas --locked`. Writes only `*-0.3.json` and patches `schema_version` to `{"const":"0.3"}`. Confirm no `expression` leaks (`tests/schema.rs` asserts this).

## Contract invariants (will break tests/schemas if missed) → `architecture/domain-contract.md`, `architecture/schema-contract.md`
- Active schema is `0.3` (`SchemaVersion::CURRENT`); binary rejects any other plan version (`UnsupportedSchema`, no migration). `tool.version` (e.g. `0.1.0`) ≠ `schema_version`. Release smoke uses a `0.3` loopback plan.
- All input structs use `deny_unknown_fields`; additive fields must be `Option`/`#[serde(default)]` + `skip_serializing_if`.
- Sole duration unit: integer microseconds (`DurationMicros`). Execution default 30s/1/0; `deadline==0`, `repetitions==0`, `retries>0` rejected — retries are contractually forbidden.
- HTTP URL must be `http/https`, no userinfo, host must equal `target.host`, port must match when `target.port` set.
- Redaction boundary (tested in `tests/contract.rs`, `tests/routed_qualification.rs`; see `architecture/render-privacy.md`): `RouteSpec` (may hold credentials) → `RouteSummary` (`{"kind":"eggress"}` only) via `summary()`; `Debug`/`Display`/human print `eggress(<redacted>)`. `DiagnosticError.message` is bounded and credential-stripped — never forward dependency `error.to_string()`/`{:?}` (URLs, headers, hop URIs) into it. Report/schema JSON must never contain `expression`.

## Engine / CLI quirks → `architecture/engine-transport.md`, `architecture/cli-automation.md`, `architecture/assertions-findings.md`, `architecture/render-privacy.md`
- Machine JSON is the contract; human text is lossy. Stdout is machine data only under `--json`/`--ndjson`; logs/diagnostics go to stderr. Single report = pretty JSON; batch `run` = compact NDJSON in input order with rewritten `execution_id=batch-N`. Single-plan `run` always emits pretty JSON. `compare --json` = pretty aggregate, else short human.
- Exit codes: `0` success, `1` negative (Failed/Unsupported status or any non-`Passed` finding — `Unavailable` also fails), `2` invalid, `3` internal, `130` interrupted. `combine_exit_codes` precedence `130>3>2>1>0`. `compare` ignores assertions (exit reflects `ReportStatus` only).
- Engine is sequential (`repetitions × probes`), no parallelism/pooling/retries. Outer finite deadline via `timeout_at`; `Ctrl+C` exits 130 with no partial report.
- `ProbeEngine::default()` is `AllowPrivate`. `--strict-target` (single-probe commands only) selects `Strict`; `run`/`compare`/`proxy` are always `AllowPrivate`.
- DNS always uses the local system resolver with `resolution_scope: "client"`, even for routed plans — never read it as remote-hop evidence.
- Direct TCP tries all resolved addresses serially (no `connect_timeout`); direct TLS tries only the first address, SNI from `--server-name` or target host, ALPN `h2,http/1.1`.
- HTTP is Eggfetch H1/H2 only (no QUIC/H3 selector); body is a 4 KiB sample with decompression off; per-phase timings are explicitly `unavailable`. HTTP 4xx/5xx are `Ok` observations until an assertion (`http_status_range`, `required_http_version/alpn/tls_version`, `max_total_micros`) fails them. Assertion string matches are exact (`HTTP/2.0`, `TLSv1_3`, `h2`).
- Eggress is listener-free client-only (`from_pproxy_uri`), byte-stream TCP dialer only, no env-proxy fallback and never falls back to direct. Bad route → generic `Protocol/HopHandshake "invalid or unsupported Eggress route"`. Only safe routed provenance is `route_hop_index` + `route_protocol`.
- `run` bounds: 4 MiB input, 256 plans, concurrency 1–64 (default 4); accepts single `ProbePlan` | `Vec<Plan>` | `{plans:[...]}`; `-` = stdin. `--fail-fast` stops admitting new plans but awaits started work and truncates output (do not assume 1:1 lines).
- CLI traps: `dns --port` is silently ignored; `tcp`/`tls`/`proxy`/`check` require `--port`; `proxy` requires `--via`; `http` target port stays `None` (URL port lives in `ProbeSpec`); `check` builds `[Dns,Tcp]` plus `Http+http-status` only with `--url`. `compare` needs direct-`Direct` + routed-`Eggress` with same target/probes, `repeat` 1–100, cold policy, nearest-rank `ceil(p*N/100)` percentiles over successful timings only.

## Release / planning boundary → `architecture/release-operations.md`, `plans/registry.md`
- Ordinary release = qualified archives + checksums + manual install; no auto-update. Per ADR-0002, Eggpack owns producer construction/CI, Eggup owns consumer update — do not add installer/manifest/self-update logic to Eggprobe. `release.yml` is `workflow_dispatch` with a `version` input validated against `^v\d+\.\d+\.\d+$`, requires tag == `eggprobe-cli` version match + `release-identity.txt` + per-archive `.sha256`. (The `v*` tag-push trigger lives in `release-skeleton.yml`, not `release.yml`.)
- Do not execute superseded `plans/.../003-shared-installer-update-integration.md`; first release is not blocked on Eggpack (M003a) or Eggup (M003b) — see `plans/registry.md` execution order. Re-check sibling (Eggfetch/Eggress/Eggpack/Eggup) interfaces at execution time; reviewed pins are not permanent.
