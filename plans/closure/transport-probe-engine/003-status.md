# Transport M003 closure

Status: closed

Source plan: `plans/implementation/transport-probe-engine/003-eggfetch-backed-http-probe.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Finding and evidence

All HTTP requests use published `eggfetch-core` 0.2.0 with HTTP/1.1 and HTTP/2,
Rustls, custom dialer support, compression disabled, bounded response bodies,
and no built-in retries/proxy environment behavior. 4xx/5xx status is retained
as successful observation. HTTP phase timings not exposed by the public observer
are explicitly listed as unavailable.

`tests/engine.rs` serves a deterministic local HTTP 503 fixture and proves the
status remains observable while an assertion fails.

## Compatibility/security

The report shape is shared with direct and routed execution. URL and method
validation occurs before I/O; body collection is capped at 4096 bytes. Locked
tests, Clippy, MSRV, dependency tree, and audit passed.

