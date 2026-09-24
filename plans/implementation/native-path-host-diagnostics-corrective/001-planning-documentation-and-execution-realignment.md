# Native Path/Host Diagnostics Corrective C001 — Planning, Documentation, and Execution Realignment

Status: ready

Repository baseline: `e131cf6e33a9c278dede1b5158069d9572b47bc8`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Related closure evidence:

- `plans/closure/native-path-host-diagnostics/001-status.md`
- `plans/closure/native-path-host-diagnostics/002-status.md`

Long-term requirements:

- `plans/002-long-term-roadmap.md#phase-8--native-path-and-host-diagnostics`
- `plans/003-planning-process.md#13-registry-rules`

Applicable ADRs:

- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

Primary work class: polish + planning/invariant hygiene

## 1. Objective

Reconcile Phase 8 planning and operator-facing documentation with the closure-backed M001/M002 implementation before additional native capabilities land.

This corrective exists because M001/M002 closed successfully but their closure exposed planning/documentation drift rather than a production-code defect:

1. the registry still serializes M004 behind the blocked M003 ICMP backend even though the subsystem dependency graph makes M003 and M004 independent after M001;
2. the registry execution-order numbering contains duplicate/stale sequence numbers and mixes current native work with old release sequencing;
3. the registry retains more closed release-corrective narrative than is useful for its stated role as a compact active control surface;
4. README/operator quick-start material does not consistently surface the implemented `route` capability or distinguish implemented native families from schema-reserved/unsupported ones;
5. `main` contains substantial post-tag Phase 8 work while Cargo still reports 0.1.1; documentation must distinguish the immutable qualified `v0.1.1` release from unreleased `main` and prevent rebuilding/publishing the changed tree under the old release identity.

No M001/M002 production behavior is being reopened.

## 2. Readiness and dependencies

Ready immediately.

M001 and M002 are closed at implementation commit `6e45e857e6a621f9e5a895ab7618a8b6dbaa6d6b` with hosted run `36051400630` green and accepted closure records.

M003 remains blocked on an ICMP backend. M004 is independently ready under the accepted dependency graph and MUST NOT wait for M003 after this corrective closes.

## 3. Invariants

- preserve all accepted M001/M002 closure evidence;
- do not rewrite ADR-0003 or historical schemas;
- do not change production Rust behavior, public schema, or dependency graph;
- registry must remain a compact active control surface rather than a duplicate closure ledger;
- blocked M003 research and ready M004 implementation may proceed in parallel;
- do not imply that post-`v0.1.1` `main` is the already-qualified 0.1.1 artifact;
- do not move or rewrite the immutable `v0.1.1` tag.

## 4. In scope

### 4.1 Registry cleanup

- fix duplicate/out-of-order numbering in `Current execution order`;
- make the current Phase 8 order explicit:
  - C001 documentation/planning cleanup;
  - M004 direct UDP may execute immediately after C001;
  - M003 ICMP backend/upstream research proceeds independently;
  - M005 remains blocked pending a truthful path backend;
  - M006 remains blocked on supporting seams;
- remove the artificial statement that M004 is held behind sequential M003 assessment;
- consolidate duplicate `Operational evidence gate` headings;
- compress stale C003-C005 release corrective detail where it merely duplicates closure records, while preserving links and any facts still needed to understand an active blocker;
- keep the closure-backed product implementation head current.

### 4.2 Native subsystem roadmap cleanup

- register this corrective as ready/active authority without altering the closed M001/M002 milestone history;
- state explicitly that M003 and M004 are parallel branches after M001;
- preserve the existing M005/M006 dependency semantics;
- retain the Phase 9 routed-datagram boundary.

### 4.3 README/operator capability reconciliation

Update user-facing docs so the current source tree is described truthfully:

- include `eggprobe route <target> --json` in the primary usage/quick-start examples;
- mention `eggprobe-native` in architecture/capability summaries where appropriate;
- distinguish:
  - implemented: DNS, TCP, TLS, HTTP, routed stream probes, target route/interface/egress-MTU evidence;
  - schema-reserved but currently typed-unsupported: ICMP echo, direct UDP, traceroute, PMTU;
- do not advertise M004 before it lands;
- keep route-candidate correlation wording explicit: it is not authoritative kernel policy-route selection.

### 4.4 Release-identity documentation

The current workspace version may remain `0.1.1` during development, but docs/planning must state:

- `v0.1.1` identifies the immutable qualified release at `53ea53d`;
- current `main` contains unreleased post-0.1.1 Phase 8 changes;
- release/package workflows MUST NOT publish or recreate changed `main` artifacts as `v0.1.1`;
- the next release qualification must select a new version/tag before artifact publication.

This corrective does not choose that future version.

## 5. Out of scope

- implementation of M003, M004, M005, or M006;
- choosing/building a new ICMP backend;
- changing Cargo package version;
- creating a new release tag;
- schema 0.5 or changes to schema 0.4;
- Eggpack/Eggup adoption;
- Phase 9 routed UDP/QUIC work.

## 6. Ordered work packages

### WP1 — Registry reconciliation

1. Re-read current registry and closure records.
2. Correct active execution order and numbering.
3. Remove the obsolete M004-behind-M003 hold.
4. Consolidate duplicated release evidence headings/narrative.
5. Register M004 as independently ready and M003 as a parallel research blocker.

### WP2 — Roadmap/status reconciliation

1. Register C001 in the native subsystem status.
2. Keep M001/M002 closed.
3. Show M003 blocked, M004 ready independently, M005/M006 blocked.
4. Confirm dependency graph and status prose say the same thing.

### WP3 — User/operator documentation

1. Add route to README and operator quick-start examples.
2. Update high-level capability text for the new native crate/route evidence.
3. Add or reconcile a concise current-native-capability statement so schema presence cannot be mistaken for backend implementation.
4. Preserve exact route evidence limitations and direct-only semantics.

### WP4 — Release identity hygiene

1. Add a concise unreleased-development note to the appropriate release/operator documentation.
2. Link qualified `v0.1.1` evidence rather than duplicating it.
3. State the future release-version gate explicitly.
4. Do not edit/move tags or package versions.

### WP5 — Verification and closure

1. Inspect the resulting diff for contradictory statuses, duplicate numbering/headings, stale M004 sequencing language, and version claims.
2. Run documentation/reference checks and repository sanity verification.
3. Write closure record and update registry/roadmap to C001 closed.

## 7. Compatibility, failure, cancellation, and resource semantics

This is a documentation/planning-only corrective. It must produce zero production behavior, schema, dependency, timeout, cancellation, or resource-management changes.

Any discovered production defect must stop this plan and receive its own corrective.

## 8. Required verification

At minimum:

```text
git diff --check
cargo metadata --locked --no-deps
cargo test --workspace --all-features --locked
```

Plus repository-content assertions:

- no duplicate execution-order numbers;
- one coherent operational-evidence section structure;
- no text saying M004 waits for M003;
- README/operator primary usage includes `eggprobe route`;
- docs distinguish qualified `v0.1.1` from unreleased `main`;
- every referenced plan/closure path exists;
- roadmap and registry agree on M001-M006 status.

## 9. Acceptance criteria

C001 closes only when:

1. registry is compact, correctly numbered, and internally consistent;
2. M004 is independently ready while M003 remains a parallel blocked research line;
3. roadmap/registry status matches closure evidence;
4. README/operator docs expose the implemented route capability and do not advertise unimplemented native backends;
5. immutable 0.1.1 release identity is clearly separated from unreleased current `main`;
6. no production/schema/dependency changes occur;
7. verification is recorded in a closure file.

## 10. Stop conditions

Stop and register separate work if cleanup reveals:

- M004 actually has a hidden hard dependency on M003;
- a documented native capability lacks the production path claimed by M001/M002 closure;
- the release workflow can publish changed `main` under an existing immutable tag without an explicit tag/version gate;
- correcting documentation requires changing schema or code behavior.

## 11. Closure evidence

Create:

`plans/closure/native-path-host-diagnostics-corrective/001-status.md`

Record the planning/docs commit range, files reconciled, before/after execution order, release-identity wording, exact verification results, and the final M003/M004 handoff state.
