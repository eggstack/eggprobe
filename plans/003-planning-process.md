# Eggprobe Planning Process

Status: canonical planning and handoff rules

This document defines how durable Eggprobe direction becomes bounded implementation work.

## 1. Authority order

When documents disagree, use this order unless a newer accepted ADR explicitly supersedes an older decision:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. accepted ADRs;
4. `plans/002-long-term-roadmap.md`;
5. subsystem roadmap;
6. milestone implementation plan;
7. current repository evidence.

Repository evidence may reveal that a plan is stale. It does not authorize silently weakening canonical invariants.

## 2. Document lifecycle

Canonical long-term documents are durable. Ordinary implementation work MUST NOT rewrite them to match temporary code.

Accepted ADRs are superseded rather than rewritten.

Subsystem roadmaps may evolve while preserving completed history and explaining material sequencing changes.

Implementation plans are baseline-specific and may be corrected, superseded, or archived.

Closure records are evidence records. Once accepted they should be immutable except for factual corrections.

## 3. Work classification

Every milestone MUST have one primary class:

- **Invariant** — protects a property that must always remain true.
- **Capability** — exposes user/operator/integration-visible behavior.
- **Infrastructure** — internal machinery required by capabilities.
- **Polish** — ergonomics, diagnostics, performance, cleanup, or documentation.

Infrastructure MUST NOT be described as completed capability before a consumer path exists.

## 4. Dependency model

Dependencies MUST be labeled:

- **hard** — implementation cannot correctly begin until closed;
- **interface** — work may proceed against a stable written/public contract;
- **soft** — work may proceed in parallel but integration depends on the other milestone;
- **operational** — code can land but release/closure needs external evidence.

A milestone is ready only when hard dependencies are closed and interface dependencies are stable.

## 5. Milestone sizing

A handoff milestone SHOULD be small enough for one implementation agent to:

- inspect the current ownership boundary;
- implement production changes;
- add deterministic fixtures;
- run focused and broad verification;
- update documentation;
- produce closure evidence;

without redesigning an unrelated subsystem.

Prefer complete vertical contracts over wide refactors with no user or integration consumer.

## 6. Greenfield rule

Because Eggprobe begins as a fresh repository, early plans MUST resist speculative framework creation.

A new abstraction is justified when it:

- represents a canonical domain concept in the long-term specification;
- is required to keep the CLI thin;
- composes Eggfetch/Eggress without duplication;
- provides a stable test seam;
- or prevents a known security/compatibility ambiguity.

Do not create plugin systems, daemons, databases, dynamic loading, or generic workflow engines merely because they might be useful later.

## 7. Implementation handoff contract

Every implementation plan MUST state:

- repository baseline;
- source roadmap;
- long-term requirements;
- applicable ADRs;
- objective and readiness;
- current evidence;
- invariants;
- in-scope/out-of-scope work;
- ordered work packages;
- failure/cancellation semantics;
- compatibility effects;
- required tests and verification;
- documentation updates;
- acceptance criteria;
- stop conditions;
- closure evidence.

Agents MUST inspect current code before editing and preserve unrelated changes.

## 8. External dependency work

Eggfetch and Eggress are sibling projects, not code to copy into Eggprobe.

If Eggprobe needs a missing observer or metadata seam:

1. prove the gap against the currently published/public API;
2. specify the minimal interface required;
3. prefer an upstream change;
4. keep Eggprobe functional with truthful unavailable/coarse evidence where reasonable;
5. do not vendor or fork general-purpose network implementations merely to avoid coordination.

A plan that requires an unpublished sibling commit MUST record that as an interface/operational dependency. Release plans SHOULD consume published crates unless an explicit temporary exception is accepted.

## 9. JSON/schema review rule

Any change affecting machine-readable public fields requires review of:

- schema version impact;
- field units;
- optional/required semantics;
- enum extensibility;
- redaction;
- stdout cleanliness;
- golden fixtures;
- CLI renderer parity.

A presentation-only CLI change does not require a schema bump if the underlying report is unchanged.

## 10. Security review rule

Network-facing milestones MUST review:

- target validation;
- local/private address policy;
- DNS rebinding implications;
- TLS verification;
- proxy credentials;
- URL/header/cookie/token redaction;
- bounded reads and error strings;
- task/socket concurrency;
- cancellation cleanup;
- untrusted certificate/protocol data;
- unsafe environment-derived proxy behavior.

## 11. Verification rule

Use deterministic local fixtures by default.

Live internet endpoints MAY be used for exploratory evidence but MUST NOT be required for routine correctness tests.

Before closure, record exact commands and outcomes. Compilation alone is insufficient evidence for a network capability.

## 12. Corrective passes

A corrective pass is a new implementation plan.

It MUST:

- reference the failed/partial closure record;
- enumerate unresolved findings;
- explain why prior evidence missed them;
- add regression evidence;
- avoid reopening unrelated closed scope.

Repeated corrective passes are a signal to revise milestone decomposition or architecture assumptions.

## 13. Registry rules

`plans/registry.md` is the compact active control surface.

It SHOULD list:

- active subsystem roadmaps;
- ready/active/blocked implementation plans;
- blockers and dependency gates;
- recently closed work.

It MUST link to source documents rather than duplicating their detailed content.

## 14. Planning review checklist

Before a plan is handed off, verify:

1. canonical references are correct;
2. architecture decisions are resolved or explicitly deferred;
3. dependency readiness is truthful;
4. scope is bounded;
5. ownership among Eggprobe/Eggfetch/Eggress is explicit;
6. JSON compatibility effects are addressed;
7. DNS/TCP/TLS/HTTP route semantics are truthful;
8. timeout/cancellation/concurrency behavior is defined;
9. security/redaction effects are defined;
10. deterministic test evidence and closure criteria are unambiguous.

If these cannot be answered, the milestone is not ready for handoff.

## 15. Planning anti-patterns

Do not:

- put transient TODOs into canonical long-term documents;
- let the CLI become a second diagnostics engine;
- parse human display strings to recover typed dependency errors;
- infer timings that are not observable;
- claim local DNS is remote proxy DNS;
- silently downgrade H3 or proxy routes;
- make retries implicit;
- mix logs with JSON stdout;
- mark capability closed from compilation alone;
- generate every possible future roadmap before it is useful.
