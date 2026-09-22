# CLI M004 closure

Status: closed

Source plan: `plans/implementation/cli-automation/004-route-comparison-and-repetition-statistics.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Finding and evidence

`compare` executes independent direct/routed plan inputs, preserves every raw
attempt, labels the current policy as cold connections, and derives count,
min/max, median/p50, and p95 from observed integer microsecond totals. Sample
count one is handled deterministically. Missing timings yield an empty stats
object rather than a fabricated comparison. No causal claim is emitted.

Locked tests, Clippy, MSRV, and audit passed.

