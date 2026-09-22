# Transport and Probe Engine — Post-Closure Corrective Addendum

Status: active; C001 closed; C002 blocked on missing published Eggress typed failure provenance

Planning baseline: `24965aa0ba2696b201e9c74531920877529821d5`

Historical evidence:

- `plans/subsystems/transport-probe-engine-roadmap.md`
- `plans/closure/transport-probe-engine/001-status.md` through `006-status.md`
- implementation commit `792ad65524a869d2ec22ba88dd9daea427c74cab`

Controlling architecture:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/000-long-term-specification.md#11-security-model`
- `plans/000-long-term-specification.md#12-timeout-repetition-and-concurrency-model`

## 1. Why this addendum exists

Post-closure audit found that the transport implementation provides a useful working prototype but several roadmap invariants and exit conditions were not actually satisfied:

- strict target policy is enforced for direct DNS/TCP/TLS but direct HTTP delegates resolution to Eggfetch without the same address policy;
- nonzero `ExecutionPolicy.retries` is accepted but ignored;
- the transport closures claim more deadline/cancellation/address-attempt evidence than the test suite demonstrates;
- direct TCP does not currently record its local socket address;
- Eggress route failures are collapsed to generic errors rather than preserving detailed kind/stage/hop/protocol provenance;
- required local TLS trust/name-mismatch and SOCKS5/CONNECT/multi-hop fixtures were not executed;
- routed H3 is absent by feature selection rather than represented as a structured unsupported request;
- one execution can allocate inconsistent execution IDs between timeout and successful paths.

Historical M001–M006 closure records remain immutable. This addendum records the corrective qualification required before release qualification.

## 2. Dependency graph

```text
Foundation corrective C002
          |
          v
Transport corrective C001
  execution/policy semantics
          |
          v
Transport corrective C002
  routed/TLS qualification
```

## 3. C001 — Execution policy, deadline, retry, and error semantics

Implementation:

- `plans/implementation/transport-probe-engine-corrective/001-policy-deadline-retry-and-error-semantics.md`

Closure:

- `plans/closure/transport-probe-engine-corrective/001-status.md`

Hard dependency: Foundation corrective C002 (closed).

## 4. C002 — Routed, TLS, and transport qualification

Implementation:

- `plans/implementation/transport-probe-engine-corrective/002-routed-tls-and-transport-qualification.md`

Closure:

- `plans/closure/transport-probe-engine-corrective/002-status.md`

Hard dependency: C001 (closed). Current blocker: published Eggress 1.0.7
collapses outbound chain failures to `EggressError::Runtime(String)`; typed
hop/stage/protocol fields are unavailable without parsing strings or copying
sibling routing internals.

## 5. Completion definition

The corrective workstream closes only when strict target policy applies to the actual connection path, unsupported/retry semantics are explicit, typed Eggress provenance is retained where the public API exposes it, and the deterministic local fixture matrix demonstrates the direct/TLS/routed claims made by the parent roadmap. C002 is stopped until the missing public Eggress diagnostic seam is available.
