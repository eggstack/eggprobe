# CLI and Automation M004 — Route Comparison and Repetition Statistics

Status: blocked

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after dependencies close.

Source roadmap:

- `plans/subsystems/cli-automation-roadmap.md#m004--direct-versus-route-comparison-and-repeated-statistics`

Hard dependencies:

- CLI M002 closed.
- Transport M005 closed for direct-versus-Eggress HTTP comparison.

Primary class: capability + polish

## 1. Objective

Add repeatable direct-versus-route diagnostics and statistically useful summaries without hiding individual attempts or manufacturing comparisons from non-equivalent evidence.

## 2. Required production changes

- `compare` operation that executes independent source plans;
- repeat-count plan policy;
- cold-versus-reused connection policy explicit in plan/report;
- per-attempt stable identifiers;
- derived min/max/median/p50/p95 as justified by sample count;
- count of successes/failures/unsupported attempts;
- comparison findings only when comparable fields exist;
- raw source reports retained or addressable from comparison result.

Do not label a difference as causal evidence merely because two routes differ.

## 3. Tests

- direct vs proxy with deterministic local latency fixture;
- one side fails/unsupported;
- missing phase timing cannot be compared;
- repeat count 1 versus larger N;
- percentile definition for small samples documented/tested;
- connection reuse disabled/enabled explicitly;
- cancellation during repetition;
- no hidden retries;
- aggregates reproduce from raw attempts.

## 4. Acceptance criteria

Statistics are derived, deterministic, and reconstructable from attempts. Comparison never erases source errors or unavailable evidence. Users can distinguish cold connection setup from pooled/warm behavior.

## 5. Stop conditions

Stop if the implementation needs to guess missing phase timings, if pooling cannot be controlled/identified, or if comparison semantics would imply unsupported causal conclusions.

## 6. Closure evidence

Create `plans/closure/cli-automation/004-status.md` with statistic definitions, fixture results, raw-to-aggregate proof, pooling policy, and failure/unavailable comparison cases.
