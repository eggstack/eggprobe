# Foundation Diagnostic Contract Corrective C001 — Route Redaction and Contract Integrity

Status: closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`

Source corrective addendum:

- `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md#4-c001--route-redaction-and-contract-integrity`

Historical source plan/closure:

- `plans/implementation/foundation-diagnostic-contract/001-rust-workspace-and-canonical-diagnostic-contract.md`
- `plans/closure/foundation-diagnostic-contract/001-status.md`

Primary class: invariant + corrective

## 1. Objective

Repair the post-closure security and contract-integrity findings without expanding into network execution.

The critical correction is to stop treating a partial hand-written Eggress URI parser as a safe redaction boundary. Until Eggress itself is linked in Transport M004, the raw route expression should be considered opaque secret-bearing input and should not be reproduced in report/debug/human output.

Also close the validated-wrapper deserialization gap and remove ambiguity over which target fields future transport code should obey.

## 2. Current evidence

At baseline, `redact_route_expression()` finds the first `://` and first subsequent `@`. It therefore does not sanitize a later hop in an expression such as:

```text
socks5://u1:p1@hop1:1080__http://u2:p2@hop2:8080
```

It also cannot faithfully parse all userinfo grammar, including passwords containing raw `@`.

The M001 fixture covers one credential-bearing authority plus a token query parameter only.

`ToolVersion` derives `Deserialize` around its inner `String`, bypassing `ToolVersion::new()` validation.

`ProbePlan` currently permits plan/probe combinations whose target authority can disagree.

## 3. Invariants

- Route secrets never appear in JSON, human output, `Debug`, `Display`, panic text, or closure fixtures.
- The pre-Eggress layer does not reimplement Eggress route parsing.
- Report route types cannot hold raw route input.
- Validated wrappers enforce validation on deserialization.
- One documented authority determines the target used by every future probe.

## 4. Scope

In scope:

- remove or neutralize partial route-expression redaction;
- safe opaque Eggress route summary before Eggress integration;
- multi-hop/raw-`@`/username-only/query-secret regressions;
- `ToolVersion` custom deserialization through validated construction;
- resolve target/probe authority structurally or through explicit `ProbePlan::validate()`;
- update fixtures/docs/closure state.

Out of scope:

- parsing Eggress chains;
- linking Eggress;
- DNS/TCP/TLS/HTTP execution;
- schema generation;
- CLI subcommands.

## 5. Required production changes

### Route boundary

Preferred correction:

- retain raw route expression only in input-only `EggressRoute`;
- make `Debug` and `Display` fully opaque, e.g. `Eggress(<redacted>)`;
- make `RouteSummary::Eggress` contain no recoverable raw expression. A fixed redacted marker and later structured safe fields are acceptable.

Do not extend the current string parser to understand more Eggress grammar.

Transport M004 will replace the opaque summary with structured/redacted data obtained from Eggress's canonical parser.

### Validated wrappers

Implement custom `Deserialize` for `ToolVersion` via `ToolVersion::new()`. Audit other validated wrappers for the same split-constructor/deserializer defect.

### Target authority

Choose and enforce one rule before transport code exists.

Preferred direction:

- `ProbePlan.target` is authoritative for logical destination identity;
- probe specs contain only probe-specific behavior, not an independent destination;
- where a probe requires additional target structure that cannot be represented safely yet (notably HTTP URL), defer final shape to Foundation M002 but reject contradictory combinations now.

If a cleaner structural target enum is adopted, keep the change bounded to the pre-1 contract and update all fixtures atomically.

## 6. Work packages

A. Replace partial Eggress redaction with opaque-safe representation.
B. Add adversarial credential fixtures: multi-hop, raw `@`, username-only, query secrets on later hops.
C. Route validated wrapper deserialization through constructors.
D. Resolve target authority and add contradictory-plan negative tests.
E. Run full M001 regression suite and reconcile docs/plans.

## 7. Required tests

- multi-hop expression leaks neither first nor later credentials;
- password containing `@` leaks no substring;
- username-only userinfo is not reproduced;
- token/password/api-key query parameters do not leak regardless of hop;
- report JSON and human renderer remain secret-free;
- empty `tool.version` JSON is rejected;
- contradictory target/probe plans fail validation if the retained model permits constructing them;
- original deterministic fixtures updated intentionally;
- no network dependencies appear in `cargo tree`.

## 8. Verification

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```

## 9. Acceptance criteria

- The discovered credential counterexamples are regression-tested and safe.
- Eggprobe no longer pretends to parse arbitrary Eggress route syntax in foundation code.
- `ToolVersion` cannot deserialize an invalid value.
- target authority is explicit enough for Transport M001 to consume without guessing.
- no Eggress/Eggfetch/Tokio dependency is introduced.
- no unresolved medium-or-higher corrective finding remains.

## 10. Stop conditions

Stop if a safe fix requires implementing Eggress parsing locally, if target-authority repair forces an HTTP/network implementation, or if the current repository has already introduced transport code that changes the ownership boundary.

## 11. Closure evidence

Create `plans/closure/foundation-diagnostic-contract-corrective/001-status.md` with exact counterexample-before/fix-after evidence, dependency tree, tests, validation review, and registry/roadmap disposition.

## 12. Handoff notes

Preserve the historical M001 closure record. This corrective supersedes only its overly broad redaction conclusion, not its implementation evidence.
