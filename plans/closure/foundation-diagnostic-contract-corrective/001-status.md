# Foundation corrective C001 closure

Status: closed

Source plan: `plans/implementation/foundation-diagnostic-contract-corrective/001-route-redaction-and-contract-integrity.md`
Source roadmap: `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md`
Repository baseline: `792ad65524a869d2ec22ba88dd9daea427c74cab`
Implementation commit: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Executive finding

The partial route parser was removed from the foundation boundary. Report,
display, and debug representations are now fixed opaque markers; raw route
input remains confined to the input plan. ToolVersion deserialization uses its
validated constructor, and ProbePlan validates target authority before I/O.

## Requirement-to-evidence matrix

| Requirement | Evidence |
|---|---|
| arbitrary multi-hop/userinfo/query secrets stay hidden | `opaque_route_boundary_handles_arbitrary_future_syntax` |
| invalid ToolVersion is rejected | `invalid_tool_versions_are_rejected_during_deserialization` |
| contradictory HTTP authority is rejected | `contradictory_http_target_is_rejected_by_plan_validation` |
| no local Eggress parser remains | `domain/route.rs` emits only `<redacted>` |

## Verification

- `cargo fmt --all -- --check` — passed.
- `cargo check --workspace --all-targets --all-features --locked` — passed.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — passed.
- `cargo test --workspace --all-features --locked` — 16 passed.
- `cargo +1.89.0 check --workspace --all-targets --locked` — passed.
- `cargo audit` — no advisories reported.

## Invariant/security review

No report type can carry raw route input. The report fixture now contains only
`<redacted>`. Target policy and later transport code consume `ProbePlan.target`.
No network dependency existed at the corrective boundary; transport dependencies
were added only in the subsequent transport implementation commit.

## Roadmap and registry disposition

C001 is closed. Foundation M002 and Transport M001 are unblocked and were
implemented in the same verified implementation sequence. Historical M001
closure was not rewritten.

