# CLI Corrective C001 — Closure

Status: closed

Implementation plan:

- `plans/implementation/cli-automation-corrective/001-batch-compare-check-and-exit-correctness.md`

Baseline: schema 0.2 Foundation C002 and Transport C001 closed.

## Delivered correction

- `check` creates an HTTP status assertion only when `--url` is supplied and
  rejects an inverted status range as code 2 before any network work.
- Batch execution uses a bounded Tokio `JoinSet`; it never schedules more than
  `--concurrency` plans, preserves input-order NDJSON, stops scheduling after
  the first negative result under `--fail-fast`, and still awaits already-started
  tasks. Results are independently parseable and receive deterministic
  `batch-N` identities.
- `compare` requires a direct first plan and an Eggress second plan with equal
  target/probe families. Direct and routed reports, attempts, counts, and
  latency distributions remain separate; deltas are null unless both sides
  have samples.
- Percentiles use nearest-rank: rank is `ceil(p*N/100)`, clamped to the first
  sample, with sorted successful probe timings only. Failed and unsupported
  attempts remain counts, not latency samples.
- CLI failures are classified as invalid (2) or internal (3), and portable
  Ctrl+C handling exits 130. Probe/assertion negatives remain code 1.

## Evidence

The CLI suite covers URL-less check, invalid assertion range, input-ordered
pure NDJSON batch output under concurrency 2, direct/routed role rejection,
side-specific comparison statistics, nonzero negative comparison exit, and
help/version/report rendering. The scheduler admission loop only spawns while
`JoinSet::len() < concurrency`; completion is collected before another plan is
admitted, so no unbounded task or output buffer is created.

## Verification

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked        # 28 passed
cargo +1.89.0 check --workspace --all-targets --locked
cargo audit
```

All commands passed. Stdout remains machine data for JSON/NDJSON modes and
diagnostics remain on stderr.

## Downstream disposition

CLI C001 is closed. Transport C002 is the remaining next qualification gate;
it is blocked by the published Eggress API’s missing typed failure provenance,
recorded in the registry and corrective addendum for reassessment.
