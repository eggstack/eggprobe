# CLI Corrective C001 — Batch, Compare, Check, and Exit Correctness

Status: closed

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Source addendum:

- `plans/subsystems/cli-automation-corrective-addendum.md#2-c001--batch-compare-check-and-exit-correctness`

Hard dependencies:

- Foundation corrective C002 closed;
- Transport corrective C001 closed.

Primary class: correctness + automation

## 1. Objective

Make the existing CLI automation surface behave as documented, without changing network ownership or inventing a second result model.

## 2. Fix `check` construction

Only add HTTP assertions when an HTTP probe is present.

Required cases:

- DNS/TCP-only check succeeds without an unavailable HTTP assertion;
- HTTP check creates status assertion;
- invalid status range (`min > max`) fails invocation/plan validation;
- route/strict policy options map exactly into the canonical plan/engine policy.

## 3. Implement actual bounded batch concurrency

`RunArgs.concurrency` must control the scheduler.

Requirements:

- at most N active plan executions;
- bounded memory/output buffering;
- deterministic per-input result identity;
- each final NDJSON record remains independently parseable;
- partial failures do not cancel unrelated plans unless `--fail-fast`;
- fail-fast stops scheduling new work but safely awaits/cancels already-started work according to the documented policy;
- output ordering policy is explicit: input order or completion order, not accidental.

Do not spawn unbounded tasks.

## 4. Correct comparison semantics

Validate the two plans:

- first input must be direct;
- second input must be routed;
- targets/probe families must be comparable or the command fails clearly.

Produce separate direct and routed aggregates:

- sample count;
- successes/failures/unsupported;
- min/max/median/p50/p95 for each comparable timing;
- optional absolute/relative delta only when both sides have comparable evidence.

Preserve every source report/attempt.

Do not merge both sides into one timing vector.

## 5. Process exit semantics

Demonstrate and enforce:

- 0: completed with successful required outcomes;
- 1: negative probe/assertion/comparison outcome;
- 2: invalid invocation/plan/input;
- 3: internal execution/serialization invariant failure;
- 130: interruption.

Do not map every runtime `Err` to usage error 2.

Add explicit signal handling/cancellation integration for Ctrl+C where portable. The core remains the source of report semantics.

## 6. Statistics correctness

Document the exact percentile definition. For small N, tests must make the chosen nearest-rank/interpolation behavior explicit.

Statistics must be reconstructable from source attempts and must not include failed/unsupported attempts as latency samples unless explicitly labeled.

## 7. Required tests

- check without URL;
- check with HTTP 503 + assertion failure;
- invalid assertion range;
- concurrency N=1 and N>1 with a barrier fixture proving max in-flight count;
- fail-fast;
- NDJSON parse/purity under concurrent execution;
- direct/routed role validation;
- separate direct/routed distributions;
- missing/noncomparable evidence;
- nonzero comparison exit when one side fails;
- Ctrl+C/interrupted exit 130 where portable;
- internal failure path exit 3.

## 8. Acceptance criteria

- every CLI option has observable semantics;
- no cosmetic concurrency option remains;
- comparison results are side-specific and reconstructable;
- DNS/TCP-only check does not fail due to an absent HTTP probe;
- exit codes match documentation;
- stdout purity is retained.

## 9. Closure evidence

Create `plans/closure/cli-automation-corrective/001-status.md` with scheduler concurrency proof, comparison fixtures/aggregates, check matrix, exit-code tests, NDJSON samples, and full verification results.
