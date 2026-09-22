# CLI and Automation M003 — Plan Files, Schema Validation, NDJSON, and Batch Execution

Status: closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after dependencies close.

Source roadmap:

- `plans/subsystems/cli-automation-roadmap.md#m003--plan-files-schema-validation-ndjson-and-batch-execution`

Hard dependencies:

- Foundation M002 closed.
- CLI M002 closed.

Primary class: capability + infrastructure

## 1. Objective

Make Eggprobe a reliable subprocess/CI/agent component through versioned plan-file input, stdin operation, bounded batch execution, and a stable NDJSON record envelope.

## 2. NDJSON decision

Use NDJSON primarily as one final record per plan execution in batch/repetition mode for the first contract. Do not mix arbitrary progress events into the same record type.

If live progress is later needed, introduce an explicitly tagged event envelope and compatibility review rather than overloading final report records.

## 3. Required production changes

- `eggprobe run <plan.json>`;
- `eggprobe run -` for stdin;
- optional batch container/manifest containing multiple canonical plans;
- schema-version validation before execution;
- bounded concurrency with deterministic plan/result identifiers;
- one parseable NDJSON final-result record per execution;
- JSON mode for single full report;
- stderr-only logs/progress;
- partial batch failures do not stop unrelated plans unless fail-fast explicitly requested.

## 4. Resource and security bounds

- maximum plan file/stdin bytes;
- maximum batch size;
- maximum concurrency;
- no unbounded buffered stdout;
- secret-bearing plan input is never echoed to diagnostics;
- malformed records fail with bounded messages.

## 5. Tests

- CLI args and equivalent plan file produce equal canonical plan;
- stdin/file paths;
- schema mismatch;
- malformed/oversized input;
- batch ordering/IDs;
- concurrency bound;
- partial success/failure;
- NDJSON line-by-line parsing;
- stdout contains no logs;
- cancellation terminates outstanding tasks cleanly.

## 6. Acceptance criteria

External programs can execute plans without terminal scraping. Every NDJSON line is independently parseable and schema-tagged. Batch resource use is bounded and failure isolation is explicit.

## 7. Stop conditions

Stop if NDJSON semantics require exposing unstable internal progress state, if schema artifacts do not match Serde behavior, or if batch execution creates an independent scheduler unrelated to core execution limits.

## 8. Closure evidence

Create `plans/closure/cli-automation/003-status.md` with plan/schema equivalence, input/output bounds, NDJSON examples, concurrency measurements, partial-failure evidence, and cancellation tests.
