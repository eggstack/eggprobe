# Foundation and Diagnostic Contract M001 Closure

Status: closed

Source implementation plan: `plans/implementation/foundation-diagnostic-contract/001-rust-workspace-and-canonical-diagnostic-contract.md`

Source roadmap: `plans/subsystems/foundation-diagnostic-contract-roadmap.md`

Repository baseline reviewed: `4cce819f22774dbf163ea56e4e5b17b07d2249e0` (planning-only repository)

Implementation commit: `f802e30` (`feat: add diagnostic contract foundation`)

## Executive finding

M001 is complete and closed. The repository now contains the minimal Rust
workspace, a typed pre-1 JSON-first plan/report contract, a redaction-safe
route boundary, deterministic fixtures, a thin CLI, and baseline CI/MSRV/audit
checks. No networking implementation or speculative Eggfetch/Eggress/Tokio
dependency was introduced.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Rust workspace with `eggprobe-core` and `eggprobe-cli` | `Cargo.toml`, two crate manifests, `cargo check --workspace --all-targets --all-features --locked` | Pass |
| Rust 1.89 MSRV | `rust-toolchain.toml`, `cargo +1.89.0 check --workspace --all-targets --locked` | Pass |
| Typed canonical plan/report contract | `crates/eggprobe-core/src/domain/` modules and Serde derives | Pass |
| Distinct schema/tool versions | `SchemaVersion`, `ToolVersion`, fixture assertions | Pass |
| Typed evidence rather than an unrestricted JSON bag | `ProbeEvidence` and typed DNS/TCP/TLS/HTTP evidence structs | Pass |
| Separate status, diagnostic error, and finding | `ProbeStatus`, `DiagnosticError`, `Finding`, report fixture | Pass |
| Integer-microsecond public timing | `DurationMicros`, `Timing`, fixture assertion for `1250` | Pass |
| Input route/report route separation | `RouteSpec` versus `RouteSummary`, `ProbeReport::from_plan` | Pass |
| Credential redaction | Debug/display/report JSON/human renderer regression test and redacted report fixture | Pass |
| CLI/core ownership | CLI depends on core; `render_report` delegates to `eggprobe_core::render`; no network dependencies in tree | Pass |
| `--help` and `--version` | `crates/eggprobe-cli/tests/cli.rs` | Pass |
| No networking in M001 | dependency tree contains only Clap, Serde, Serde JSON, thiserror, and their transitive support crates | Pass |
| Baseline quality gates | `.github/workflows/ci.yml`, local verification below | Pass |

## Final workspace and crate layout

```text
Cargo.toml
rust-toolchain.toml
crates/
├── eggprobe-core/
│   ├── src/domain/{error,finding,plan,probe,report,route,target,timing,version}.rs
│   ├── src/{lib,render}.rs
│   └── tests/{contract.rs,fixtures/{plan,report}.json}
└── eggprobe-cli/
    ├── src/{lib,main}.rs
    └── tests/{cli,render}.rs
```

The CLI is a parser and renderer adapter. There are no subcommands or fake
network operations before the transport milestones land.

## Dependency tree and rationale

The locked tree has 32 crate dependencies. Direct production dependencies are:

- `serde` for typed contract serialization;
- `serde_json` for the canonical JSON renderer and fixtures;
- `thiserror` for validated input/version errors;
- `clap` for the minimal CLI help/version surface;
- the local `eggprobe-core` dependency from the CLI.

The tree contains no Eggfetch, Eggress, Tokio, Hickory, Rustls, UUID, tracing,
schema-generation, table, or color dependency. The transitive Clap styling
crates are a consequence of Clap's default feature set and are not used to
implement a renderer contract.

## Fixture and redaction evidence

`crates/eggprobe-core/tests/fixtures/plan.json` is a representative input plan
containing an intentionally credential-bearing Eggress route. The report
fixture contains only:

```text
http://alice:[REDACTED]@example.net:8080?token=[REDACTED]
```

`contract.rs` verifies deterministic pretty-JSON equality and round trips for
both fixtures. It also asserts that the password and token do not occur in
route `Debug`, route `Display`, report JSON, or human-rendered output. Invalid
target JSON and unknown route discriminators fail during deserialization.

## Exact verification results

| Command | Outcome |
|---|---|
| `cargo fmt --all -- --check` | Pass |
| `cargo check --workspace --all-targets --all-features --locked` | Pass |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Pass; no warnings |
| `cargo test --workspace --all-features --locked` | Pass; 9 tests across 8 suites |
| `cargo +1.89.0 check --workspace --all-targets --locked` | Pass |
| `cargo tree --locked` | Pass; locked dependency tree reviewed, no network stack |
| `cargo audit` | Pass; advisory database loaded 1,261 advisories and 32 locked crate dependencies scanned with no reported vulnerabilities |
| `git diff --check` | Pass before implementation commit |

The CI workflow repeats format, strict Clippy, locked tests, and the Rust
1.89.0 check. Audit remains a documented/release verification command rather
than a network-dependent CI gate.

## Invariant review

- Machine-readable core types are canonical; the CLI has no second result model.
- `RouteSpec` may contain input credentials, while `RouteSummary` cannot hold
  the raw expression and is the only route type in reports.
- Probe status, diagnostic error, findings, warnings, and unavailable facts are
  separate fields/types.
- Public durations are integer microseconds; no `Instant` or wall-clock value
  is serialized.
- Schema version `0.1` and tool version `0.1.0` are distinct.
- Enum values use explicit Serde discriminators and an `other` category where
  the initial taxonomy needs additive growth; unknown required discriminators
  fail cleanly.
- `unsafe_code` is forbidden at the workspace and crate levels.
- No unrestricted evidence map or network implementation was added.

## Failure, cancellation, and resource review

M001 performs no network operations, creates no background tasks, and has no
durable state. Target/version validation and malformed required discriminators
fail deterministically. Rendering is pure and only appends to a local string;
it does not mutate reports or perform I/O.

## Compatibility and schema review

The contract is explicitly pre-1 (`0.1`) and is not declared stable for a
published major release. Schema and tool versions are independent. Optional
fields use deliberate omission behavior in fixtures; durations are integer
microseconds. Generated JSON Schema is correctly deferred to foundation M002.

## Security/redaction review

No secret is needed to execute M001. The route input type is explicitly
input-only in the report construction API, and custom `Debug`/`Display` paths
redact password userinfo and token-like query parameters without an Eggress
dependency. The fixture covers password and token leakage. Target validation
rejects empty, oversized, whitespace-containing, malformed-label, and port-zero
inputs before future execution code can use them.

Residual security work for future network milestones remains outside this
closure: private-address policy, DNS rebinding, TLS verification, bounded
network reads, cancellation cleanup, and proxy/environment policy.

## Documentation and operations review

`README.md` states the pre-release status, JSON-first ownership, currently
implemented foundation-only capability, MSRV, and exact verification commands.
Crate/module documentation explains ownership. `.github/workflows/ci.yml`
provides stable quality and Rust 1.89 lanes. The MIT license and workspace
repository metadata are present.

## Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| High | None | — |
| Medium | None | — |
| Low | Human rendering is intentionally minimal and Clap's default style support is transitive | Deferred by scope; no contract impact |

No unresolved medium-or-higher M001 finding remains.

## Roadmap and registry disposition

M001 is marked closed in the foundation roadmap and this closure record is
linked from the registry. The following future handoffs are now dependency-
ready for planning:

- foundation M002 schema/compatibility fixture gate;
- transport M001 direct route/DNS/TCP primitives;
- release M001 CI/MSRV/audit/release skeleton.

CLI M001 remains blocked until real primitive probes exist. Transport M002–M006,
CLI M002–M004, and release M002–M004 remain blocked by their respective
downstream dependencies. No future milestone is claimed complete by this
foundation closure.

## Final disposition

Closed. The repository is ready for the next handoff plans, with no networking
scope mixed into foundation M001.
