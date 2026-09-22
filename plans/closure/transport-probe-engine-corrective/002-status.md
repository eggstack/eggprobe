# Transport Corrective C002 — Closure

Status: closed

Implementation plan:

- `plans/implementation/transport-probe-engine-corrective/002-routed-tls-and-transport-qualification.md`

Roadmap/addendum:

- `plans/subsystems/transport-probe-engine-roadmap.md`
- `plans/subsystems/transport-probe-engine-corrective-addendum.md`

Baseline reviewed: `2f01ab52fb1abe0e424cd7062b6dcc39d8e5298f`.

Implementation commits:

- `da40a1a49772b9d06367088a343195a5199d5d77` — Eggress 1.0.8 typed route errors, transport/schema changes, deterministic route/TLS fixtures, and documentation.
- `9d2dc74cc8f7c81b8acef55d0f84cd67f018acc3` — routed TLS cancellation evidence.

## Executive finding

Eggress 1.0.8 resolves the upstream interface blocker: its published outbound
connector exposes detailed connect errors, including kind, stage, hop index,
and protocol. Eggprobe now uses that API and preserves the exposed facts
through both native routed probes and Eggfetch's custom dialer without parsing
display strings. Local fixtures qualify Eggress SOCKS5/HTTP CONNECT routes,
multi-hop behavior, TLS verification, H1/H2, deadlines, and cancellation.
No public endpoint is used as correctness evidence.

The route topology remains:

```text
ProbeEngine ── Eggfetch custom Dialer (HTTP only) ──┐
                                                   ├─ Eggress 1.0.8 outbound connector ── hop chain ── target
ProbeEngine TCP/TLS ───────────────────────────────┘
```

Eggprobe starts no proxy listener. Eggress listeners in tests belong only to
the local in-process fixture.

## Requirement-to-evidence matrix

| Requirement | Evidence |
|---|---|
| Typed failure provenance | First-hop refusal and later-hop authentication failure assert normalized kind, stage, zero-based hop, and protocol. Eggfetch route failure retains the typed Eggress error as its source. |
| Route success and no direct fallback | SOCKS5 and HTTP CONNECT local fixtures reach their target; first-hop failure asserts the target listener receives no connection. |
| Authentication safety | SOCKS5 and HTTP CONNECT rejection fixtures retain typed provenance; serialized report assertions confirm credentials are absent. |
| Chain behavior | Two-hop SOCKS5 success reaches the local target; second-hop rejection identifies hop index 1. |
| TLS verification and metadata | In-process trusted-host, name-mismatch, and untrusted-root fixtures; negotiated version/cipher and ALPN present/absent assertions. Routed TLS verifies `localhost` SNI/name against a trusted fixture certificate. |
| Eggfetch origin TLS and H1/H2 | Direct and Eggress-routed Eggfetch HTTP/2 requests use `https://localhost`; a fixture-local dialer maps only the routed socket destination to loopback, preserving the logical origin for Eggfetch TLS/SNI. Routed HTTP/1.1 uses a local HTTP CONNECT fixture. |
| Deadline and cancellation | Routed TCP and Eggfetch connection deadlines produce structured deadline evidence; standalone and routed TLS cancellation tests observe transport closure. |
| DNS semantics | Routed DNS probe output explicitly marks the answer scope as `client`; it does not claim the answer is the remote proxy's resolution. |
| H3 disposition | No plan/CLI surface requests H3, and the custom dialer is byte-stream based. H3 is documented as unavailable; no claim is made that a structured H3-only unsupported path exists. |

The only deliberately unavailable TLS evidence is a bounded peer-certificate
summary; the report does not claim to provide one. Eggress typed source
metadata unavailable from the public error API is not inferred from text.

## Production implementation evidence

- Eggress direct and routed TLS/TCP connects use
  `connect_tcp_timeout_detailed()` with remaining execution-deadline budget.
- Eggfetch's `RouteDialer` preserves `OutboundConnectError` as the source of
  its `DialError`; HTTP normalization downcasts that typed source and maps
  stable Eggprobe kinds/stages plus optional hop/protocol fields.
- Eggress zero-based hop indexes and protocol labels are represented in
  optional report fields. Human-readable messages remain bounded and do not
  include route expressions or credentials.
- DNS evidence declares `resolution_scope: client`. Report/plan schemas were
  advanced to 0.3; the 0.2 schema artifacts remain available unchanged.
- HTTP protocol qualification is H1/H2. H3/QUIC is not represented as a
  selectable transport and is explicitly documented as unavailable.

## Verification

All commands below passed against the implementation commits:

```text
rtk cargo fmt --all -- --check
rtk cargo check --workspace --all-targets --all-features --locked
rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
rtk cargo test --workspace --all-features --locked              # 46 passed, 11 suites
rtk cargo +1.89.0 check --workspace --all-targets --locked
rtk cargo audit                                                # 215 dependencies; no advisories reported
rtk cargo run -p eggprobe-core --example generate-schemas --locked
rtk cargo tree --locked -i eggress-embed                      # 1.0.8
rtk cargo tree --locked -i eggress-core                       # 1.0.8 family
rtk git diff --check
```

Schema generation was repeated; both runs produced identical SHA-256 values:

- `schemas/plan-0.3.json`: `87afcb6fc3afffef6610238d2f018c5d676f4cb5954738d9c2780d0fb716f7f6`
- `schemas/report-0.3.json`: `9a4ce4c84aca48cbeb6ebaa6d8bf388a35f294184fd85d4a4ff9869961050ebd`

## Invariant, failure, and compatibility review

- TLS verification remains enabled by default; negative trust/name cases fail
  closed.
- Route failures never trigger direct fallback. Execution and connection
  deadlines bound work, and cancellation drops in-flight direct and routed
  streams. Production introduces no listener or detached probe task.
- Route credentials are excluded from diagnostic messages and serialized
  reports. No secrets, session keys, or peer-certificate contents are emitted.
- Plan/report schema version 0.3 records client DNS scope and optional typed
  route provenance. Version 0.2 schema files are retained; fixtures, CLI
  expectations, docs, and generated schemas align on 0.3.
- No medium-or-higher transport qualification finding remains. H3 selection,
  remote-resolver answers, and certificate summaries are explicit limitations,
  not implied capabilities.

## Documentation and downstream disposition

README, operator documentation, schema documentation, transport roadmap,
corrective addendum, registry, and this implementation plan describe the
qualified H1/H2 and Eggress 1.0.8 behavior and the H3/DNS limitations.

- Transport corrective C002 is closed; Transport C001 and C002 are both
  complete, so the transport corrective gate for Release M004 is removed.
- Release M004 remains blocked: Release corrective C001 is conditionally
  closed pending hosted evidence from a dispatch against an existing release
  tag, and Release M002 still needs that operational qualification. The repo
  has no existing release tag, so no safe tag dispatch is available in this
  round. M003 remains blocked on a stable published `eggup` interface and is
  not advertised as an M004 prerequisite unless updater claims are added.
- No listed implementation plan remains to execute. Do not promote Release
  M004 or M003 until their respective external dependencies are satisfied.

The registry and subsystem roadmaps were updated to reflect this closure and
the remaining independent release blockers.
