# Native C001 Closure — Planning, Documentation, and Execution Realignment

Status: closed

Source implementation plan:

- `plans/implementation/native-path-host-diagnostics-corrective/001-planning-documentation-and-execution-realignment.md`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Planning baseline (the planning registration commit):

- `9eee6a76a0ed01f3bf53d09eb9e7e498e660b969` — `plans: register native diagnostics documentation cleanup`

Reviewed repository baselines:

- `e131cf6e33a9c278dede1b5158069d9572b47bc8` — repository baseline named in the plan (post-M001/M002 close);
- `6e45e857e6a621f9e5a895ab7618a8b6dbaa6d6b` — closure-backed product implementation head (Native M001/M002);
- `53ea53d14560c150d0ebc10de83eca10de37202d` — qualified release of record (`v0.1.1`).

This corrective is documentation/planning only; no production Rust, public
schema, dependency, or contract behavior changed during its execution.

## Executive finding

C001 is complete. The registry is compact and internally consistent, the
dependency graph and milestone status now agree, M004 is independently
ready after M001/M002 (no longer held behind M003), the implemented
`eggprobe route` capability is surfaced in the primary usage and quick
start, the `eggprobe-native` crate is named in architecture/capability
summaries, schema-reserved but unimplemented native families are clearly
distinguished from implemented ones, and the immutable qualified
`v0.1.1` release is cleanly separated from the unreleased `main` tree.

No M001/M002 production behavior was reopened. No schema was bumped.
No tag or Cargo package version was moved.

## Files reconciled

| File | Change |
|---|---|
| `plans/registry.md` | Consolidated two duplicate `Operational evidence gate` headings into one `Release evidence gate`; added a `Release identity hygiene` section; fixed duplicate `1.` / `4.` execution-order numbering; removed the "held behind sequential M003 assessment" hold on M004; updated the M004 dependency-ready handoff note to state that direct UDP is independent of M003 after M001; tightened the planning-hygiene bullets to call out the compact-control-surface role and the immutable-tag rule. |
| `plans/subsystems/native-path-host-diagnostics-roadmap.md` | Updated the M001→M003/M004 dependency prose to call out parallel branches after M001, not a sequence; updated the M004 milestone status row to "ready (parallel to M003 after M001)" and to remove any implicit M003 dependency in its blocker column. |
| `README.md` | Added `eggprobe route <target> --json` to the primary usage block; added new `Native diagnostics` section distinguishing implemented primitives (DNS/TCP/TLS/HTTP, `eggprobe route`) from schema-reserved but currently typed-unsupported families (ICMP, direct UDP, traceroute, PMTU); added new `Release identity` section pointing to the qualified `v0.1.1` release and warning that the changed `main` tree MUST NOT be republished under that tag. |
| `docs/operator.md` | Added `eggprobe route <target> --json` to the quick-start block; added a new `Native diagnostics` section mirroring the README distinction; added a new `Release identity` section restating the `v0.1.1` qualified-release fact and the no-republish-under-that-tag rule. |
| `architecture/overview.md` | Updated the diagram caption from `schemas/*.json (0.3 active)` to `schemas/*.json (0.4 active)`; extended the `Probes:` tool bullet to name `eggprobe-native` and the route/interface/source/MTU evidence; extended the `Schemas:` tool bullet to note that schema `0.4` reserves additional native families that currently dispatch as typed unsupported results. |
| `architecture/release-operations.md` | Updated the MSRV/toolchain line to state that the workspace `version` is currently `0.1.1` while `main` carries unreleased Phase 8 native work, and to call out that the qualified release of record remains `v0.1.1` (tag at `53ea53d`). |

Before/after execution order (registry `Current execution order`):

- before: `1.`, `1.`, `2.`, `3.`, `4.`, `4.`, `5.`, `6.`, `7.`, `8.`, `9.`, `10.`, `11.`, `12.`, `13.` with two duplicate `1.` entries (C005 vs release-candidate selection) and two duplicate `4.` entries ("Do not wait for M003a/M003b…" vs "M003a remains blocked…");
- after: `1.`, `2.`, `3.`, `4.`, `5.`, `6.`, `7.`, `8.`, `9.`, `10.`, `11.`, `12.`, `13.`, `14.`, `15.` with C001 inserted between the closed Native M002 step (now item 9) and the post-C001 M004 step (now item 11), and M003/M004 explicitly parallel (items 11 and 12).

Before/after release-evidence gate (registry):

- before: two `## Operational evidence gate` headings (one tagged "M004 closed", one untagged) covering overlapping M004 / C004 / C001 / M002 release evidence;
- after: one `## Release evidence gate` heading covering the same evidence compactly, plus a new `## Release identity hygiene` heading linking the `v0.1.1` qualification and the no-republish rule.

Before/after dependency prose (native roadmap §6):

- before: "M003/M004 hard-depend on M001 and soft-depend on M002. M005 hard-depends on M001 plus backend qualification; M003/M004 are soft dependencies. M006 hard-depends on M001 and should follow M002/M004/M005 so route/datagram/path test seams exist.";
- after: "M003/M004 hard-depend only on M001; they soft-depend on M002 and are parallel branches after M001, not a sequence. M005 hard-depends on M001 plus backend qualification; M003/M004 are soft dependencies. M006 hard-depends on M001 and should follow M002/M004/M005 so route/datagram/path test seams exist. The Phase 9 routed-datagram boundary remains separate from this graph."

## Acceptance criteria (plan §9)

1. ✅ Registry is compact, correctly numbered, and internally consistent
   (duplicate `Operational evidence gate` headings consolidated; duplicate
   `1.` / `4.` execution-order numbering fixed; compact `Release evidence
   gate` + `Release identity hygiene` sections replace verbose
   release-corrective narrative; planning-hygiene bullets tightened).
2. ✅ M004 is independently ready while M003 remains a parallel blocked
   research line (registry dependency-ready handoff note rewritten;
   native roadmap §6 + milestone status row updated; registry execution
   order items 11 and 12 state the parallelism explicitly).
3. ✅ Roadmap/registry status matches closure evidence (M001/M002 remain
   `closed` with the `6e45e85` closure head and `plans/closure/native-path-host-diagnostics/{001,002}-status.md`
   cited; M003 `blocked`, M004 `ready`, M005/M006 `blocked` consistently
   in both files).
4. ✅ README/operator docs expose the implemented route capability and do
   not advertise unimplemented native backends (`eggprobe route` is in
   the primary usage and quick-start; new `Native diagnostics` sections
   distinguish implemented primitives from schema-reserved families and
   name M003/M004/M005/M006 status truthfully; `eggprobe-native` is
   named in the architecture overview's probe bullet).
5. ✅ Immutable 0.1.1 release identity is clearly separated from
   unreleased current `main` (registry `Release identity hygiene`,
   README `Release identity`, operator `Release identity`,
   architecture/release-operations MSRV/toolchain note all point to
   `v0.1.1` at `53ea53d` and forbid republishing the changed `main`
   under that tag).
6. ✅ No production/schema/dependency changes occur (this corrective is
   documentation/planning only; no Rust, schema, dependency, timeout,
   cancellation, or resource-management change was made).
7. ✅ Verification is recorded in this closure file.

## Required verification (plan §8)

```text
git diff --check                                       PASS
cargo metadata --locked --no-deps --format-version 1   PASS (workspace metadata resolves, version 0.1.1)
cargo test --workspace --all-features --locked         PASS (all suites green; native substrate tests included)
```

Repository-content assertions:

- ✅ no duplicate execution-order numbers (registry `Current execution
  order` is now strictly 1..15);
- ✅ one coherent release-evidence section structure (`Release evidence
  gate` + `Release identity hygiene` replace the two `Operational
  evidence gate` headings);
- ✅ no text saying M004 waits for M003 (the only remaining references
  describe the historical hold that this corrective removed or the
  parallel-branch fact that replaces it);
- ✅ README/operator primary usage includes `eggprobe route` (both
  `README.md` Usage and `docs/operator.md` Quick start now list it);
- ✅ docs distinguish qualified `v0.1.1` from unreleased `main`
  (README, operator, registry, and architecture/release-operations all
  state this explicitly);
- ✅ every referenced plan/closure path exists (each path cited in the
  reconciled files was checked against the working tree before this
  closure was written; no missing paths);
- ✅ roadmap and registry agree on M001–M006 status (both list M001/M002
  closed, M003 blocked, M004 ready, M005/M006 blocked).

## Compatibility, failure, cancellation, resource, security review

- Compatibility/schema: no schema change; historical schema files remain
  untouched; `0.4` remains the active contract.
- Failure/cancellation/resources: no engine, CLI, or runtime change;
  bounded retry/deadline/cancellation semantics are unchanged.
- Security/privacy: redaction boundary is not touched; `RouteSpec` →
  `RouteSummary` semantics are unchanged; no new credential paths were
  introduced.
- Operations: no release or packaging change; the documented rule that
  the next qualification must select a new version is planning-only.

## Downstream disposition

- **M003 (ICMP echo):** remains blocked on a safe backend. C001 does not
  change its readiness gate or any registered blocker.
- **M004 (direct UDP):** may execute immediately after C001. The plan
  `plans/implementation/native-path-host-diagnostics/004-direct-udp-service-diagnostics.md`
  is already `ready`; nothing in C001 changed its scope or tests.
- **M005 (traceroute):** remains blocked on a truthful backend; unchanged.
- **M006 (active PMTU):** remains blocked on test seams + PMTU controls;
  unchanged.
- Release corrective C001's operational gate is unaffected.
- Eggpack/Eggup M003a/M003b paths are unaffected.
- The post-`v0.1.1` `main` branch is documented as carrying unreleased
  Phase 8 work; the next release qualification must select a new
  version before artifact publication.

## Roadmap and registry updates

- `plans/registry.md`: native C001 row updated from `ready` to `closed`
  with closure head equal to this corrective's commit; the registry's
  active-workstreams row, dependency-ready row, and execution-order
  items are already consistent with the closure.
- `plans/subsystems/native-path-host-diagnostics-roadmap.md`: §10.1
  C001 row already carries the "ready" marker; the status header
  remains accurate; the §6 dependency prose and the §12 milestone
  status row are now consistent with the closure.
- No other subsystem roadmaps or ADRs were touched; ADR-0003 and the
  historical schema files remain immutable per the invariants.
