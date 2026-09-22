# Transport M004 closure

Status: closed

Source plan: `plans/implementation/transport-probe-engine/004-eggress-listener-free-route-core.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Finding and evidence

Eggress 1.0.7 `OutboundConnector::from_pproxy_uri` and `connect_tcp` are the
canonical route parser and listener-free connector. Eggprobe wraps the
Eggress stream only to satisfy Eggfetch's public Tokio stream trait, retains
peer/hop metadata where exposed, and maps failures to opaque stable messages.
No direct fallback is attempted after route failure. Report/debug/display route
values remain opaque for every chain expression.

The source ownership boundary is visible in `engine.rs`; dependency-tree and
audit verification passed. Full SOCKS/CONNECT multi-hop fixtures remain an
operational follow-up requiring the sibling testkit, not a local protocol copy.

