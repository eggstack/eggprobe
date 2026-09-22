# Foundation and Diagnostic Contract Milestone 001 — Rust Workspace and Canonical Diagnostic Contract

Status: ready for handoff

Repository baseline: `4cce819f22774dbf163ea56e4e5b17b07d2249e0` (planning-only repository; no production source)

Source roadmap:

- `plans/subsystems/foundation-diagnostic-contract-roadmap.md#m001--rust-workspace-and-canonical-diagnostic-contract`

Long-term requirements:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#5-canonical-execution-model`
- `plans/000-long-term-specification.md#6-canonical-report-contract`
- `plans/000-long-term-specification.md#9-error-status-and-assertion-semantics`
- `plans/000-long-term-specification.md#10-serialization-and-automation`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

Primary class: infrastructure + invariant

## 1. Objective

Create the minimal production Rust workspace and canonical diagnostic data contract that every later Eggprobe capability will consume.

At closure, the repository must have a compiling `eggprobe-core` library, a thin `eggprobe` CLI binary, explicit schema/tool version fields, deterministic JSON fixtures, a redaction-safe route/report boundary, baseline CI/MSRV/security checks, and no network implementation.

This milestone must establish enough domain structure that the first networking milestone can implement DNS/TCP directly into real report types rather than redesigning output around already-landed code.

## 2. Why this milestone is ready

Hard dependencies:

- none.

Stable interface dependencies:

- Rust 1.89 is the current Eggstack floor used by Eggfetch 0.2.0 and Eggress 1.0.7.
- JSON-first/core ownership is accepted in ADR-0001.
- Later Eggfetch/Eggress dependencies are documented but do not need to be linked in M001.

The repository contains planning only, so no production compatibility or migration is required.

## 3. Current implementation evidence

At baseline:

- no root `Cargo.toml`;
- no Rust source;
- no CI;
- no release workflow;
- no schemas;
- no tests beyond planning documents;
- no public Eggprobe API to preserve.

The planning baseline already records:

- canonical product scope;
- diagnostic terminology;
- phase ordering;
- transport ownership;
- subsystem decomposition.

M001 should not re-litigate those decisions.

## 4. Invariants that must not regress

1. Machine-readable types are canonical; CLI presentation is downstream.
2. `eggprobe-cli` must not contain networking code or an independent result model.
3. Probe status, diagnostic error, and assertion finding are separate concepts.
4. Plan route input and report route summary are separate types so credential-bearing input is not copied blindly into reports.
5. Public durations serialize as integer microseconds.
6. Schema version and Eggprobe binary version are distinct.
7. Unknown/future enum growth must be considered explicitly; avoid exhaustive public coupling where later additions are expected.
8. Sensitive fields must have safe `Debug`/`Display` behavior or remain in input-only types that never flow into report rendering.
9. No `serde_json::Value` bag may become the primary representation of probe evidence merely to avoid typed design.
10. No network socket, HTTP client, proxy client, DNS resolver, TLS connector, or background runtime belongs in this milestone.

## 5. Scope

### In scope

- root Rust workspace;
- Rust toolchain/MSRV declaration;
- `eggprobe-core` crate;
- `eggprobe-cli` crate producing the `eggprobe` binary;
- workspace lint/profile baseline;
- typed plan/report contract;
- JSON serialization/deserialization;
- deterministic fixture construction;
- minimal pure renderers or renderer trait/module sufficient to prove CLI/core separation;
- `--version` and `--help`;
- baseline CI;
- format/lint/test/audit documentation/scripts if useful;
- crate/module documentation;
- initial README sufficient to explain status and developer verification.

### Explicitly out of scope

- DNS/TCP/TLS/HTTP execution;
- Tokio runtime unless required solely by an already-accepted core interface (prefer deferring it);
- Eggfetch/Eggress dependencies in production code;
- schema generation with `schemars` (M002);
- NDJSON;
- plan-file CLI execution;
- assertions evaluation;
- human table styling;
- shell completions;
- release binaries/installers;
- telemetry;
- configuration files;
- persistent storage;
- plugins/bindings.

## 6. Required production changes

### Workspace

Create a root workspace with resolver 3 and at least:

```text
crates/eggprobe-core
crates/eggprobe-cli
```

Use:

- edition 2021 unless current Eggstack convention has intentionally moved;
- `rust-version = "1.89"`;
- workspace license/repository/homepage metadata;
- `unsafe_code = "forbid"` or stricter unless a later probe has a reviewed platform requirement;
- pedantic Clippy baseline with explicit targeted allowances rather than global suppression.

Do not create empty future crates.

### Core/domain

Create focused modules rather than one giant `lib.rs`. Exact file names may vary, but ownership should be recognizable:

```text
domain/
  plan
  target
  route
  probe
  report
  error
  finding
  timing
  version
render/      # only if renderer contracts live in core
redact/      # if a dedicated safe-display helper is justified
```

Do not create a generic framework layer above these domain concepts.

### Canonical plan types

At minimum represent:

- `SchemaVersion`;
- `ProbePlan`;
- `TargetSpec`;
- `RouteSpec`;
- `ProbeSpec`;
- `ExecutionPolicy`;
- `AssertionSpec` or an explicit placeholder type if assertions are intentionally deferred.

Recommended initial route distinction:

- `Direct`;
- `Eggress { ... }` as an input-only form whose raw route expression is never reused as report display without redaction.

If the plan can contain credential-bearing route text, its `Debug` implementation MUST redact it.

### Canonical report types

At minimum represent:

- tool provenance/version;
- schema version;
- execution identifier;
- normalized target summary;
- redacted route summary;
- overall status;
- ordered probe results;
- timing values;
- structured diagnostic error;
- findings;
- warnings/unavailable/unsupported facts.

Probe-result data SHOULD remain typed by probe kind. Do not use an unrestricted `HashMap<String, Value>` as the main evidence contract.

The initial evidence structures may contain only fields already well understood from the long-term design. Pre-1 evolution remains possible until M002 freezes fixtures.

### Status/error taxonomy

Define non-string categories sufficient for later normalization.

The initial diagnostic error kind SHOULD include at least:

- `dns`;
- `timeout`;
- `connection_refused`;
- `network_unreachable`;
- `host_unreachable`;
- `authentication`;
- `tls`;
- `protocol`;
- `policy`;
- `io`;
- `unsupported`;
- `internal`;
- `other`.

The stage vocabulary SHOULD include at least:

- `resolution`;
- `direct_connect`;
- `hop_connect`;
- `hop_handshake`;
- `tls_handshake`;
- `request`;
- `response_headers`;
- `body`;
- `deadline`;
- `other`.

These enums should be designed for additive growth. Do not promise that every dependency error maps one-to-one.

### Timing

Represent duration in microseconds using an integer type in JSON.

Do not expose `std::time::Instant` or platform timestamps in public serialization.

If wall-clock start time is included, use an explicit well-defined UTC representation and keep it separate from monotonic durations. It is acceptable to defer wall-clock timestamps in M001 if not needed.

### Route redaction boundary

Create a testable distinction between:

- plan route input;
- report route summary.

A fixture using a route containing username/password or token-like material MUST prove the report/debug rendering does not contain the secret.

Do not depend on Eggress yet merely for redaction in M001; create only the minimal Eggprobe boundary required to avoid accidental propagation. M004 transport work can replace/augment route parsing with Eggress's canonical redacted representation.

### CLI

`eggprobe-cli` must depend on `eggprobe-core`.

M001 only needs:

- `eggprobe --help`;
- `eggprobe --version`;
- enough renderer/test wiring to prove reports are presented from core types.

Do not invent placeholder network subcommands.

If a sample/debug report command is used for tests, keep it explicitly hidden/test-only rather than presenting it as a supported user command.

### Dependencies

Keep M001 narrow. Likely production dependencies:

- `serde`;
- `serde_json`;
- `thiserror`;
- `clap` in CLI.

Add another crate only with a concrete M001 use.

Do not add Eggfetch, Eggress, Tokio, Hickory, Rustls, UUID, tracing, schema-generation, table/UI, or color crates preemptively unless implementation evidence establishes an immediate need.

If execution IDs need generation, first consider a small deterministic/local representation owned by the caller/report constructor. Avoid pulling a large randomness/UUID dependency merely to populate fixtures.

### CI and repository quality

Add a minimal GitHub Actions workflow for supported host coverage appropriate to a fresh Rust project.

At minimum qualify:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

Add an MSRV compile/check lane using Rust 1.89.0 or the repository's established exact-floor convention.

Security audit may use `cargo audit` as a documented/release check. Do not add audit ignores without evidence.

A full cross-platform release matrix belongs to the release subsystem, not M001.

### Documentation and static guards

Add enough README/developer documentation to state:

- project is pre-release;
- JSON-first/core architecture;
- current implemented capability truthfully;
- MSRV;
- verification commands.

A lightweight source guard MAY be added to prevent `eggprobe-cli` from depending directly on Eggfetch/Eggress/network crates once those dependencies arrive, but do not create brittle grep machinery before there is a dependency to guard.

## 7. Ordered work packages

### WP1 — Initialize the minimal workspace

Intent:

Create a clean Rust project structure that matches Eggstack release targets without speculative crates.

Required changes:

- root workspace manifest;
- toolchain/MSRV;
- core/CLI manifests;
- root license/readme metadata;
- basic source files.

Acceptance evidence:

- metadata resolves;
- `cargo check --workspace --all-targets`;
- Rust 1.89 check.

### WP2 — Implement canonical version/target/route/plan types

Intent:

Make input intent explicit before output/result design.

Required changes:

- schema/tool version representation;
- target types;
- route input types;
- probe specifications;
- execution policy;
- custom redacted debug/display for any credential-bearing route field.

Acceptance evidence:

- positive JSON round trips;
- malformed/unknown required field tests;
- route secret debug/redaction test.

### WP3 — Implement report/error/timing/finding types

Intent:

Create the machine contract that transport work will populate.

Required changes:

- report envelope;
- probe result envelope;
- typed evidence variants/structs;
- status;
- diagnostic error kind/stage;
- timing microseconds;
- findings/warnings/unavailable state;
- redacted route summary.

Acceptance evidence:

- deterministic expected JSON fixture;
- no secrets in report fixture;
- evidence/error/finding remain separately representable.

### WP4 — Prove CLI/core separation

Intent:

Ensure CLI is an adapter rather than a second implementation.

Required changes:

- Clap root command;
- `--help`/`--version`;
- pure renderer invocation path for a test fixture or library-facing renderer test;
- no networking dependencies.

Acceptance evidence:

- CLI integration tests;
- dependency tree/source inspection proving no networking stack in CLI.

### WP5 — Establish quality gates

Intent:

Make the greenfield baseline difficult to degrade accidentally.

Required changes:

- format/clippy/test workflow;
- MSRV lane;
- lockfile policy;
- audit instructions or workflow where appropriate.

Acceptance evidence:

- local or CI-equivalent commands pass;
- no warnings hidden with blanket allowances.

### WP6 — Documentation and closure preparation

Intent:

Leave the repository ready for transport M001 handoff.

Required changes:

- README truthfully describes only implemented foundation;
- crate docs explain ownership;
- update planning status after implementation;
- prepare closure evidence.

Acceptance evidence:

- active docs do not claim DNS/TCP/TLS/HTTP support before those milestones.

## 8. Failure, cancellation, restart, and contention semantics

M001 executes no network operations and has no durable state.

Required semantics are therefore limited to:

- malformed JSON/invalid domain values fail deterministically;
- serialization must not panic on valid types;
- redaction must not depend on network parsing;
- renderer failure must not mutate report data;
- no background tasks exist.

Future cancellation fields MAY exist but no fake cancellation engine should be implemented.

## 9. Compatibility and migration

There is no prior Eggprobe production API.

This milestone creates a pre-1 contract. Breaking refinements are allowed before M002 closure but must be deliberate and accompanied by fixture updates.

Do not declare schema major version 1 stable during M001 unless the implementation team intentionally accepts that compatibility burden. Prefer a pre-1 schema identifier until M002 freezes the initial published contract.

No storage/config migration exists.

## 10. Required tests

### Focused unit tests

- schema/tool version parse/display;
- target validation;
- route redaction;
- timing unit conversion/serialization;
- error kind/stage serialization;
- status/finding separation;
- probe-specific evidence serialization.

### Integration tests

- representative full `ProbePlan` JSON round trip;
- representative full `ProbeReport` JSON round trip/golden fixture;
- CLI `--help`;
- CLI `--version`;
- machine renderer stdout fixture if exposed.

### Security and negative tests

- credential-bearing route input does not leak through `Debug`, route summary, report JSON, or human renderer;
- malformed target/route inputs fail cleanly;
- unknown required discriminators do not panic.

### Compatibility tests

- canonical duration fields serialize as integer microseconds;
- schema version and tool version are both present and distinct;
- omitted optional fields have deliberate serialization behavior.

No restart/contention/network tests are required in M001.

## 11. Required verification commands

Use the repository's final command names, but closure should include at least the equivalent of:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```

If `cargo audit` is unavailable in the implementation environment, record that as not run/environmental rather than pass.

Do not add a dependency merely to satisfy a verification command.

## 12. Documentation updates

Expected:

- root `README.md`;
- crate-level docs;
- this implementation plan status;
- `plans/subsystems/foundation-diagnostic-contract-roadmap.md`;
- `plans/registry.md`;
- closure record at completion.

Do not modify canonical long-term documents merely to document implementation mechanics.

## 13. Acceptance criteria

- Root Rust workspace exists and declares MSRV 1.89.
- `eggprobe-core` and `eggprobe-cli` compile.
- CLI depends on core and contains no networking implementation.
- Canonical plan/report types serialize and deserialize deterministically.
- Schema version and tool version are distinct fields.
- Route input and report route summary are distinct types.
- A credential fixture proves report/debug/human output redaction.
- Probe status, diagnostic error, and assertion finding are separately represented.
- Durations serialize as integer microseconds.
- Typed evidence is used instead of an unrestricted JSON map as the primary contract.
- No speculative Eggfetch/Eggress/Tokio/network dependency is added.
- Format, strict Clippy, tests, and MSRV check pass.
- Audit is clean or any environmental limitation is recorded.
- README accurately states that networking probes are not implemented yet.
- No unresolved medium-or-higher M001 finding remains.

## 14. Stop conditions

Stop and report rather than improvise if:

- Rust 1.89 cannot support the minimal chosen dependencies;
- a proposed type shape would require committing to unknown transport behavior not covered by the long-term documents;
- safe redaction would require storing raw secrets in report types;
- the CLI can only be implemented by creating a second domain model;
- a dependency requires native/system components inconsistent with the intended portable foundation;
- repository state has acquired production code since this baseline that materially changes the greenfield assumptions;
- implementation scope expands into real networking or schema-generation M002.

## 15. Closure evidence required

Create:

- `plans/closure/foundation-diagnostic-contract/001-status.md`

The closure record must include:

- implementation commit(s);
- final workspace/crate layout;
- dependency tree summary and rationale;
- MSRV evidence;
- representative plan/report JSON fixtures;
- redaction fixture evidence;
- CLI/core ownership evidence;
- exact format/check/clippy/test/audit results;
- README/doc reconciliation;
- unresolved findings by severity;
- final disposition;
- roadmap/registry updates.

## 16. Handoff notes

- Preserve the planning hierarchy and accepted ADR.
- Do not add network capabilities opportunistically.
- Keep dependencies minimal.
- Prefer explicit structs/enums over generic maps.
- Use local deterministic fixtures only; M001 needs no internet.
- Do not mark M002 or transport M001 ready until this milestone closes.
- If type design reveals a true cross-subsystem architectural conflict, write/propose an ADR rather than burying the decision in implementation.
