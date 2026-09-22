# Foundation and Diagnostic Contract Roadmap

Status: active; M001 ready for handoff

Long-term references:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#5-canonical-execution-model`
- `plans/000-long-term-specification.md#6-canonical-report-contract`
- `plans/000-long-term-specification.md#9-error-status-and-assertion-semantics`
- `plans/000-long-term-specification.md#10-serialization-and-automation`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md#phase-0--repository-foundation-and-diagnostic-contract`

Related ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

## 1. Purpose and ownership boundary

This subsystem establishes the Rust workspace, canonical diagnostic domain types, schema/version rules, error/finding separation, renderer boundary, and baseline verification machinery.

It owns:

- `ProbePlan` and `ProbeReport` domain contracts;
- target, route, probe, observation, timing, status, error, finding, assertion, and provenance types;
- serialization/version semantics;
- core crate versus CLI crate ownership;
- baseline repository quality gates.

It does not own DNS, TCP, TLS, HTTP, proxy execution, terminal styling beyond a minimal smoke renderer, release packaging, or external upstream changes.

## 2. Work classification

### Invariants

- Machine-readable report data is canonical.
- CLI code does not own network execution.
- Display strings are not parsed as machine contracts.
- Durations have one canonical serialized unit.
- Error/finding/probe-status meanings remain distinct.
- Sensitive values cannot enter report types through unrestricted debug/display paths.

### Capabilities

- Parse and serialize a versioned no-network `ProbePlan`.
- Serialize a versioned `ProbeReport`.
- Render a report through a thin CLI adapter.

### Infrastructure

- Cargo workspace;
- `eggprobe-core`;
- `eggprobe-cli`;
- Serde/schema foundations;
- error/status enums;
- deterministic fixture helpers;
- CI/MSRV/audit scaffolding.

### Polish

- pretty human renderer;
- shell completions;
- extensive examples.

## 3. Non-goals

- no network sockets in M001;
- no Eggfetch/Eggress adapter in this subsystem;
- no TUI;
- no daemon;
- no plugin system;
- no persistent database;
- no Python/Node/FFI bindings.

## 4. Current state

The repository was fresh at planning bootstrap on 2026-09-22 and contained no production code.

Sibling dependency evidence reviewed during planning:

- Eggfetch main commit `8959ca890ee34f4cf456aed648315322f1e83ef7` publishes the coordinated 0.2.0 line and exposes the core HTTP/custom-dialer surfaces Eggprobe intends to consume later.
- Eggress main commit `cd18c19c391e72b93d994acf60298e872fce5f0c` is on the 1.0.7 workspace line and exposes listener-free outbound chain execution.
- CodeGG planning baseline `239c51d19a63e8cf369a952b78d1e55092f4653b` supplied the planning conventions mirrored here.

There is no compatibility burden from prior Eggprobe releases.

## 5. Target architecture

Initial workspace:

```text
eggprobe/
|-- Cargo.toml
|-- rust-toolchain.toml
|-- crates/
|   |-- eggprobe-core/
|   `-- eggprobe-cli/
|-- schemas/
|-- tests/
|-- docs/
`-- plans/
```

The core crate owns domain types and later execution. The CLI depends on core.

A minimal conceptual boundary:

```rust
pub async fn execute(plan: ProbePlan) -> ProbeReport
```

M001 may stub execution for a no-network/version probe if needed, but MUST NOT create a second CLI-owned result model.

Schema generation belongs to M002 after the canonical type shape is proven.

## 6. Dependency graph

```text
M001 workspace + diagnostic contract
   |
   +--> M002 schema artifact + compatibility fixtures
   |
   `--> transport-probe-engine M001
```

- M001: no hard dependencies.
- M002 hard-depends on M001.
- Transport subsystem has a hard dependency on M001 because network results must target a stable report vocabulary.

## 7. Milestones

### M001 — Rust workspace and canonical diagnostic contract

Class: infrastructure + invariant.

Objective:

Create the minimal workspace and typed JSON-first contract that all later networking work consumes.

Dependencies:

None.

Deliverable boundary:

- workspace/MSRV/lint baseline;
- core/CLI crate split;
- canonical types;
- JSON round-trip fixtures;
- minimal CLI parse/render smoke;
- baseline CI/security checks.

User or operator value:

A stable automation shape exists before networking semantics expand.

Exit conditions:

- canonical types compile and round-trip;
- CLI has no networking implementation;
- JSON stdout can be validated deterministically;
- Rust 1.89 build path is qualified;
- closure record has no unresolved medium-or-higher finding.

Deferred work:

Real probes, generated schema, NDJSON, advanced CLI rendering.

### M002 — Published schema and compatibility fixture gate

Class: invariant + infrastructure.

Objective:

Generate and freeze the first explicit pre-1 JSON Schema/fixture contract from canonical Rust types.

Dependencies:

M001 closed.

Deliverable boundary:

- schema-generation tool or test;
- golden plan/report fixtures;
- forward-compatible enum/optional-field conventions;
- schema compatibility review procedure;
- fixture checks in CI.

User or operator value:

Automation consumers can validate inputs/outputs independently.

Exit conditions:

- generated schema is reproducible;
- schema source of truth is Rust types;
- compatibility fixtures fail on accidental breaking changes;
- docs state pre-1 compatibility policy.

Deferred work:

Major-version compatibility tooling for a future 1.0 contract.

## 8. Cross-cutting requirements

### Storage and migration

No persistent storage is required.

### Protocol and compatibility

Schema version is independent from tool version. Pre-1 still requires explicit versioning.

### Security and authorization

No credentials should be needed in M001/M002. Redaction-safe types must be structurally possible before route credentials are introduced.

### Concurrency, cancellation, and recovery

No network concurrency in this subsystem. Future execution types must not preclude cancellation metadata.

### Observability and audit

Reports carry execution/provenance identifiers. No daemon audit log.

### Performance and resource use

Domain serialization must remain bounded and avoid storing arbitrary response bodies.

### Documentation and operations

Document type semantics, stdout/stderr contract, and compatibility rules with the code.

## 9. Verification strategy

- unit tests for enum/field semantics;
- JSON plan/report round trips;
- deterministic golden fixtures;
- negative deserialization tests;
- CLI stdout purity tests;
- MSRV compile;
- strict Clippy/format/test/audit.

## 10. Risks and decision points

- Exact schema-generation crate should be selected in M002 from current maintained options; likely `schemars`, but M001 should not overcommit if it adds no immediate value.
- UUID versus monotonic/local identifiers for execution IDs is an implementation detail as long as serialized identifiers are stable strings and do not leak secrets.
- Human renderer details are deliberately deferred.

No additional ADR is currently required.

## 11. Completion definition

The subsystem closes when the repository has a tested canonical diagnostic model, generated machine schema/fixtures, a thin CLI boundary, and compatibility rules suitable for downstream transport work.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 Rust workspace and canonical diagnostic contract | ready | `plans/implementation/foundation-diagnostic-contract/001-rust-workspace-and-canonical-diagnostic-contract.md` | pending `plans/closure/foundation-diagnostic-contract/001-status.md` | — |
| M002 published schema and compatibility fixture gate | blocked | not yet written | pending `plans/closure/foundation-diagnostic-contract/002-status.md` | hard: M001 closure |
