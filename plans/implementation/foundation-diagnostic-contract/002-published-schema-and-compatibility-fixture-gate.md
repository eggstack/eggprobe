# Foundation and Diagnostic Contract M002 — Published Schema and Compatibility Fixture Gate

Status: blocked

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh implementation baseline after corrective C001 closes.

Source roadmap:

- `plans/subsystems/foundation-diagnostic-contract-roadmap.md#m002--published-schema-and-compatibility-fixture-gate`

Hard dependency:

- corrective C001 in `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md`

Primary class: invariant + infrastructure

## 1. Objective

Freeze the first intentional pre-1 machine contract after the redaction/validation corrective, generate JSON Schema from canonical Rust types, and make accidental compatibility drift fail CI.

## 2. Scope

In scope:

- finalize target/route/probe/report shapes after C001;
- choose and pin a maintained schema-generation crate, expected `schemars` if MSRV/security fit;
- derive/generate plan and report schemas;
- checked-in generated artifacts under `schemas/`;
- golden valid and invalid fixtures;
- documented additive/breaking-change policy;
- CI regeneration/diff check;
- examples for external validators.

Out of scope:

- network probes;
- NDJSON event schema;
- 1.0 stability promise;
- generic schema migration framework.

## 3. Required design decisions

Before generating artifacts:

- `ProbePlan.target` authority must be unambiguous;
- route input versus safe report route must remain distinct;
- optional fields and defaults must be intentional;
- non-exhaustive enum growth strategy must be documented;
- unknown required discriminators should fail clearly;
- public units remain integer microseconds;
- schema version and tool version remain distinct.

Generated schemas are outputs. Rust types remain source of truth.

## 4. Work packages

A. Re-audit canonical types after C001.
B. Add schema derives/generator with the narrowest dependency surface.
C. Generate plan/report schema artifacts deterministically.
D. Expand golden fixtures: minimal, representative, invalid, future-unknown behavior.
E. Add CI guard that regeneration produces no diff.
F. Document pre-1 compatibility policy and schema-consumer expectations.

## 5. Tests and verification

- schema generation deterministic across two runs;
- every committed valid fixture validates against generated schema;
- invalid fixtures fail for the expected reason;
- Serde round trip agrees with schema;
- redacted route schema cannot require/expose raw report credentials;
- Rust 1.89, strict Clippy, workspace tests, audit.

## 6. Acceptance criteria

- `schemas/plan-0.x.json` and report counterpart are reproducible;
- accidental field/enum/unit changes break a compatibility test;
- target authority is represented consistently;
- documentation distinguishes pre-1 evolution from stable-major compatibility;
- no network behavior added.

## 7. Stop conditions

Stop if schema generation requires raising MSRV without review, if generated schema cannot accurately represent Serde behavior, or if C001 is not closed.

## 8. Closure evidence

Create `plans/closure/foundation-diagnostic-contract/002-status.md` with schema hashes, generator command, fixture matrix, compatibility review, dependency delta, and final subsystem disposition.
