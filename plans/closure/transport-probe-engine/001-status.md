# Transport M001 closure

Status: closed

Source plan: `plans/implementation/transport-probe-engine/001-direct-route-dns-and-tcp-primitives.md`
Source roadmap: `plans/subsystems/transport-probe-engine-roadmap.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Finding and evidence

`ProbeEngine` owns finite-deadline direct DNS and TCP execution. DNS uses the
system resolver with a bounded 32-answer list. TCP preserves address attempt
count, selected peer, refusal/timeout classification, and cancellation-by-drop.
`TargetPolicy::AllowPrivate` supports diagnostics; `Strict` rejects private and
loopback destinations before dialing. Environment proxies are not consulted.

Deterministic evidence is in `tests/engine.rs`: local TCP success, strict
loopback rejection, and report status/error assertions.

## Verification

Locked format/check/Clippy/test/MSRV/audit commands passed; workspace tests:
16 passed. No public network endpoint is required.

## Review

The report distinguishes client/system DNS from routed DNS; no remote resolver
address is inferred. Outer deadlines are finite and no implicit retries exist.

