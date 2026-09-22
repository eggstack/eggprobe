# Transport M005 closure

Status: closed

Source plan: `plans/implementation/transport-probe-engine/005-eggfetch-over-egress-routed-http.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Finding and evidence

Routed HTTP uses one in-process Eggfetch `Dialer` backed by Eggress outbound
streams. Eggfetch retains HTTP framing, origin TLS, SNI, hostname verification,
and response processing; Eggress retains hop semantics. No listener or direct
fallback is created. HTTP/3 is not compiled and is therefore explicit by
absence rather than silent downgrade.

Local HTTP evidence, dependency-tree review, locked tests, MSRV, Clippy, and
audit passed. The common report schema retains route-specific unavailable facts.

