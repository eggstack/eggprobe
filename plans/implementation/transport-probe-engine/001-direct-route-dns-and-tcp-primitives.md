# Transport and Probe Engine M001 — Direct Route, DNS, and TCP Primitives

Status: closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after corrective C001 closure.

Source roadmap:

- `plans/subsystems/transport-probe-engine-roadmap.md#m001--direct-route-dns-and-tcp-primitives`

Hard dependencies:

- foundation corrective C001 closed.

Primary class: capability + infrastructure

## 1. Objective

Add the first real network execution path: explicit direct-route policy, system-resolution DNS diagnostics, and cancellable finite-deadline TCP connection diagnostics that populate the canonical report types.

## 2. Ownership

Eggprobe owns these direct diagnostic primitives. Do not add Eggfetch/Eggress yet.

Use Tokio for async socket execution. Prefer platform/system resolver behavior for the initial DNS probe; introduce Hickory only if it is required to expose a truthful capability rather than to replace the OS path.

## 3. Required production changes

- async `ProbeEngine::execute(ProbePlan)` or equivalent core-owned runner;
- direct route implementation;
- explicit `TargetPolicy` with CLI-default diagnostics policy permitting loopback/private destinations and a strict/embed-friendly policy;
- DNS probe with A/AAAA results, elapsed time, resolver mode, bounded answer count;
- TCP probe with IPv4/IPv6 attempts, selected peer/local address, finite deadline, cancellation-safe drop;
- stable OS/I/O error normalization;
- attempt records for multiple addresses without hiding failures;
- no automatic retry beyond declared plan policy.

Do not claim the system resolver server address if the portable API cannot observe it.

## 4. Work packages

A. Add Tokio/runtime ownership in core without leaking runtime logic into CLI.
B. Define target policy and direct dial interface.
C. Implement DNS probe through system-resolution semantics.
D. Implement TCP attempts/address ordering and deadline behavior.
E. Map results/errors into canonical evidence.
F. Add deterministic local fixtures and docs.

## 5. Tests

- localhost IPv4 and IPv6 success where host supports each family;
- connection refused;
- deadline/timeout using deterministic local fixture or controlled unroutable seam without flaky internet;
- cancellation cleanup;
- DNS success and invalid/nonexistent name classification;
- bounded address lists;
- local/private policy allow/deny modes;
- no environment proxy influence;
- report timing/error fields and attempt ordering.

## 6. Acceptance criteria

- `ProbeSpec::Dns` and TCP execute through core;
- every operation has a finite outer deadline;
- direct CLI policy can diagnose LAN/loopback;
- embedded strict policy can reject private targets;
- no hidden retry/fallback;
- machine reports distinguish client DNS from route DNS;
- no medium-or-higher finding.

## 7. Stop conditions

Stop if target authority remains ambiguous after C001, if deterministic timeout tests require public internet, or if system-resolution semantics are misrepresented as richer DNS evidence than actually observable.

## 8. Closure evidence

Create `plans/closure/transport-probe-engine/001-status.md` with platform behavior, fixture topology, deadline/cancellation evidence, error matrix, target-policy evidence, and dependency tree.
