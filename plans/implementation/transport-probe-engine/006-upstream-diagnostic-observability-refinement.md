# Transport and Probe Engine M006 — Upstream Diagnostic Observability Refinement

Status: blocked

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after M003-M005 consumer paths exist.

Source roadmap:

- `plans/subsystems/transport-probe-engine-roadmap.md#m006--upstream-diagnostic-observability-refinement`

Dependencies:

- interface evidence from working M003-M005 paths.

Primary class: polish + infrastructure

## 1. Objective

Close only the diagnostic evidence gaps demonstrated by working Eggprobe consumers, preferably through narrow reusable Eggfetch/Eggress public observer seams.

This milestone is not required for an initial truthful release when evidence can be represented as unavailable.

## 2. Candidate upstream gaps

Eggfetch:

- per-request DNS start/result/failure events;
- per-address TCP attempt/connect/failure timing;
- origin TLS start/negotiated/failure timing;
- sanitized peer certificate summary on the same connection.

Eggress:

- successful per-hop connect timing;
- successful hop handshake timing;
- sanitized hop identity/protocol observer events.

Do not assume all candidates are still missing. Re-audit current releases first.

## 3. Required approach

For each claimed gap:

1. demonstrate the exact missing field in Eggprobe against a deterministic fixture;
2. verify no current public API exposes it;
3. define the minimal typed upstream observer contract;
4. implement upstream in the owning repository if approved;
5. publish/consume a released version;
6. keep Eggprobe compatible with absence via explicit unavailable fields where reasonable.

Do not fork or vendor upstream networking code.

## 4. Event design constraints

Observer events must be:

- non-secret;
- bounded;
- correlated to request/attempt/hop;
- non-blocking or explicitly backpressured;
- monotonic-timing friendly;
- non-exhaustive/additive where public;
- semantically consistent across transport families before claiming parity.

## 5. Tests

- before/after fixture proves new evidence;
- no duplicate event/timing ownership;
- no credential/certificate secret leakage;
- observer disabled has negligible semantic effect;
- cancellation/error paths emit coherent terminal state;
- old coarse report remains decodable.

## 6. Acceptance criteria

Every new timing/evidence field comes from a real observer, not inference. Upstream ownership is preserved. Eggprobe remains truthful when a provider/feature does not expose the observation.

## 7. Stop conditions

Stop if the only justification is cosmetic timing detail, if upstream change would materially redesign a sibling project, or if a metric cannot be made semantically consistent.

## 8. Closure evidence

Create `plans/closure/transport-probe-engine/006-status.md` documenting each researched gap, upstream commit/release if any, before/after evidence, compatibility impact, and deferred unavailable facts.
