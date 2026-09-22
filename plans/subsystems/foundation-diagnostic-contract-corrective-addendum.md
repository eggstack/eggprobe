# Foundation Diagnostic Contract — Post-Closure Corrective Addendum

Status: active; C001 closed; C002 closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`

Related historical evidence:

- `plans/closure/foundation-diagnostic-contract/001-status.md` — M001 historical closure.
- `plans/implementation/foundation-diagnostic-contract/001-rust-workspace-and-canonical-diagnostic-contract.md` — original M001 plan.
- `plans/subsystems/foundation-diagnostic-contract-roadmap.md` — parent roadmap.

Controlling architecture:

- `plans/000-long-term-specification.md#4-architectural-principles`
- `plans/000-long-term-specification.md#11-security-model`
- `plans/001-terminology-and-domain-model.md`
- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

## 1. Why this addendum exists

Post-closure review found that the temporary M001 Eggress-route redactor does not safely cover Eggress's full multi-hop credential grammar. The implementation scans only the first authority, so credentials in later `__`-separated hops can survive into `RouteSummary`, JSON, `Debug`, and human rendering. Raw `@` in a password and username-only userinfo also expose weaknesses in the partial parser.

Two lower-severity contract-integrity issues were also found before schema freeze:

- `ToolVersion` custom construction rejects empty strings, but derived Serde deserialization can bypass that validation.
- plan target authority is underspecified because target host/port can conflict with probe-local ports or HTTP URL.

The historical M001 record remains immutable evidence of what was tested. This addendum records the newly discovered gaps and gates schema freeze/transport execution until they are corrected.

## 2. Corrective invariant

Before downstream network work proceeds:

1. raw Eggress route expressions must be treated as sensitive input;
2. no hand-written partial Eggress parser may be relied on to sanitize arbitrary chain syntax;
3. every public validated wrapper must enforce the same invariant on direct construction and deserialization;
4. plan target authority must be explicit enough that transport code cannot choose between contradictory fields.

## 3. Dependency graph

```text
historical M001 closure
        |
        v
C001 route-redaction + contract-integrity correction
        |
        +--> foundation M002 schema freeze
        `--> transport M001 DNS/TCP
```

Release M001 may proceed independently because it does not consume route semantics.

## 4. C001 — Route redaction and contract integrity

Class: invariant + corrective.

Implementation:

- `plans/implementation/foundation-diagnostic-contract-corrective/001-route-redaction-and-contract-integrity.md`

Closure target:

- `plans/closure/foundation-diagnostic-contract-corrective/001-status.md`

Exit conditions:

- no multi-hop/userinfo secret can reach report/debug/human output through the pre-Eggress boundary;
- `ToolVersion` deserialization uses the same validation as construction;
- plan target/probe authority is explicit and enforced or structurally unambiguous;
- fixtures cover the discovered counterexamples;
- no networking dependency is introduced merely to repair the foundation;
- strict workspace verification passes.

## 5. C001 disposition

C001 closed in `plans/closure/foundation-diagnostic-contract-corrective/001-status.md`.
Foundation M002 and the initial transport sequence were subsequently implemented.
The historical M001/C001/M002 records are retained as evidence.

## 6. C002 — Structural report safety and authority normalization

Post-closure review at repository baseline
`24965aa0ba2696b201e9c74531920877529821d5` found two additional contract
issues:

- `RouteSummary::Eggress { expression: String }` can itself deserialize or
  construct arbitrary secret-bearing text even though ordinary report
  construction inserts `"<redacted>"`;
- the local HTTP authority parser does not fully normalize scheme-default ports
  and URL edge cases.

Implementation:

- `plans/implementation/foundation-diagnostic-contract-corrective/002-structural-report-safety-and-authority-normalization.md`

Closure target:

- `plans/closure/foundation-diagnostic-contract-corrective/002-status.md`

Status: implementation complete; closure evidence is recorded in
`plans/closure/foundation-diagnostic-contract-corrective/002-status.md`.

C002 was the hard gate for the transport and CLI corrective work because it
advances the pre-1 contract to an explicit corrected schema before those
correctives add new evidence against it.

## 7. Current downstream disposition

Historical closure records are not rewritten. Foundation C002 is the current
contract authority. Transport corrective C001 and CLI corrective C001 are
unblocked by this closure. Release packaging corrective C001 is independent.
