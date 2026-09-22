# Foundation M002 closure

Status: closed

Source plan: `plans/implementation/foundation-diagnostic-contract/002-published-schema-and-compatibility-fixture-gate.md`
Source roadmap: `plans/subsystems/foundation-diagnostic-contract-roadmap.md`
Repository baseline/implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Executive finding

The pre-1 contract is schema-described with Schemars derives, checked-in plan
and report artifacts, explicit opaque route summaries, typed assertions, and
deterministic generation tests. Rust types remain authoritative.

## Evidence

- `crates/eggprobe-core/src/schema.rs` generates plan/report schemas.
- `schemas/plan-0.1.json` and `schemas/report-0.1.json` are checked in.
- `crates/eggprobe-core/tests/schema.rs` proves deterministic generation.
- Contract fixtures round-trip through Serde and retain integer microsecond units.
- Unknown discriminators and invalid targets fail before execution.

## Verification and compatibility review

`cargo fmt`, locked workspace check, strict Clippy, workspace tests, Rust 1.89
check, `cargo tree --locked`, and `cargo audit` all passed. Route input and
report route summary remain distinct; report schemas require `<redacted>` for
Eggress summaries. This remains pre-1 additive evolution, not a 1.0 promise.

## Roadmap disposition

Foundation M002 is closed. Transport and CLI consumers may rely on the schema
and report contract.

