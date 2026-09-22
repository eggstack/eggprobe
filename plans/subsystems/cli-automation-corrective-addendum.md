# CLI and Automation — Post-Closure Corrective Addendum

Status: active; C001 closed

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Historical evidence:

- `plans/subsystems/cli-automation-roadmap.md`
- `plans/closure/cli-automation/001-status.md` through `004-status.md`
- implementation commit `792ad65524a869d2ec22ba88dd9daea427c74cab`

## 1. Why this addendum exists

The CLI prototype exposes the intended command families, but post-closure review found behavior that does not yet satisfy the original automation contracts:

- `check` always creates an HTTP-status assertion even when no HTTP probe is requested;
- `run --concurrency` validates a value but executes plans sequentially;
- `compare` combines direct and routed timings into one aggregate instead of producing separate comparable distributions/deltas;
- `compare` does not validate route roles and always returns process code 0;
- interruption/internal error handling does not yet demonstrate the advertised 130/3 behavior;
- batch/compare tests do not demonstrate the closed milestone claims.

## 2. C001 — Batch, compare, check, and exit correctness

Implementation:

- `plans/implementation/cli-automation-corrective/001-batch-compare-check-and-exit-correctness.md`

Closure:

- `plans/closure/cli-automation-corrective/001-status.md`

Hard dependencies:

- Foundation corrective C002;
- Transport corrective C001.

The transport routed qualification C002 is a soft/qualification dependency for routed comparison fixtures; CLI C001 may implement its logic before that closes but final routed comparison closure evidence should reuse the qualified route fixture when available.
