# Transport Corrective C001 — Closure

Status: closed

Implementation plan:

- `plans/implementation/transport-probe-engine-corrective/001-policy-deadline-retry-and-error-semantics.md`

Baseline: `24965aa0ba2696b201e9c74531920877529821d5`, with Foundation C002
schema 0.2 closed before transport changes.

## Delivered correction

- Direct HTTP installs an Eggfetch custom dialer that resolves and policy-checks
  every address immediately before the actual TCP connect. Strict policy no
  longer relies on a separate preflight lookup and cannot fall back to the
  native Eggfetch resolver.
- `ExecutionPolicy.retries > 0` is rejected as an explicit schema-0.2 plan
  validation error. No hidden retry loop remains.
- One outer execution timeout covers the complete DNS/TCP/TLS/HTTP operation.
  Timeout results retain the single execution ID and contain a structured
  failed probe with `timeout`/`deadline` evidence.
- Direct TCP reports `TcpStream::local_addr()` when available and retains
  deterministic address-attempt counts.
- Direct I/O categories include refused, timeout, network-unreachable, and
  host-unreachable mappings. Eggfetch request failures use its typed detailed
  error API and bounded static messages.
- HTTP status responses remain observations, so a 503 can be asserted against
  without being erased as a transport failure.

## Evidence matrix

| Invariant | Deterministic evidence |
|---|---|
| Actual strict HTTP dial path | loopback HTTP plan under `TargetPolicy::Strict` returns `policy` before a request |
| Permissive local HTTP | existing 503 local listener test receives and reports status 503 |
| Retry disposition | plan validation test rejects retries=1 as `UnsupportedRetries(1)` |
| Outer deadline | stalled local listener yields a structured failed HTTP probe at deadline |
| Cancellation/drop | timeout owns the complete inner future; no spawned probe tasks are used |
| Local/peer TCP evidence | local listener success asserts non-empty local address and peer evidence |
| Stable execution ID | success and timeout reports use the ID allocated at `execute` entry |
| Assertion validation | invalid inclusive status ranges are rejected before execution |

No public endpoint is used as correctness evidence.

## Verification

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked        # 23 passed
cargo +1.89.0 check --workspace --all-targets --locked
cargo tree --locked
cargo audit
```

All commands passed. The dependency baseline remains `eggfetch-core 0.2.0`
and `eggress-embed 1.0.7`; no sibling implementation was copied.

## Downstream disposition

- Transport C001 is closed.
- Transport C002 is ready and may proceed against the corrected direct-policy
  and deadline semantics.
- CLI corrective C001 is unblocked; its final routed-comparison evidence should
  reuse Transport C002 fixtures where practical, but its logic may proceed now.
- Release corrective C001 remains independently in operational qualification;
  its hosted run is recorded separately because this repository has no existing
  release tag.
