# Eggprobe Active Planning Registry

This file is the compact control surface for active interim planning. Detailed
requirements remain in canonical documents, ADRs, subsystem roadmaps,
implementation plans, closure records, corrective addenda, and Git history.

## Canonical direction

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`
- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

## Status vocabulary

- **proposed** — roadmap/plan exists but is not approved for execution.
- **ready** — hard dependencies/interfaces are satisfied.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency/evidence requirement prevents progress.
- **closing** — implementation landed; closure evidence is being gathered.
- **closed** — closure record accepted.
- **conditionally closed** — implementation is substantially complete with
  named evidence outstanding.
- **corrective required** — historical closure exists but a later finding must
  close before current qualification.
- **superseded** — replaced by a newer plan/decision.
- **archived** — retained only for traceability.

## Current baselines

Last closure-backed product implementation head:

`6e45e857e6a621f9e5a895ab7618a8b6dbaa6d6b` (Native M001/M002)

C003 planning baseline:

`cf857fc24579263611396d4d88a1ef622601d612`

C003 implementation/closure head:

`23da04ec32d3981aa0cb899d4d5b1d120e66f75e`

C003 corrected its original CRLF and audit-tool failures, but hosted CI run
`36008924756` at the closure head exposed a later Windows-only HTTP engine
fixture failure. Windows job `107664241734` passes the deterministic contract
tests and then fails `http_status_is_observed_even_when_assertion_fails`.
C004 owned that post-closure finding and is now closed.

C004 planning baseline:

`23da04ec32d3981aa0cb899d4d5b1d120e66f75e`

C004 implementation/closure head:

`04c84d3e4721707839fe286b6c94ecfdd568eac0`

C004 qualifying hosted run `36015806725` (Ubuntu `107687917563`, macOS
`107687917768`, Windows `107687917787`, MSRV `107687917366`, audit
`107687917580`) is fully green with no production change. Closure:

`plans/closure/release-operational-qualification-corrective/004-status.md`

C005 planning baseline:

`ff789d4c02d112ffb8909ba90f198ceebce1b6ad`

C005 implementation/closure head:

`2760b8b8750272ad448272fa0878ca57bbd2d880` (tag `v0.1.0`)

C005 qualifying hosted packaging run `36030784594` on `2760b8b` is fully
green across all five targets with independently verified artifact
checksums, identical release identity, and host-native smoke on all
host-native targets (Linux aarch64 build-qualified only). Closure:

`plans/closure/release-operational-qualification-corrective/005-status.md`

Release ownership realignment planning baseline:

`a96911854b73e055ab67ea2f7c1592796f031d58`

The earlier broad implementation commit
`792ad65524a869d2ec22ba88dd9daea427c74cab` remains historical evidence; its
post-closure findings have been handled by the registered corrective sequence.

## Active workstreams

| Workstream | Status | Current work | Authority |
|---|---|---|---|
| Foundation diagnostic contract | closed corrective | C002 closed | `plans/subsystems/foundation-diagnostic-contract-corrective-addendum.md` |
| Transport/probe engine | closed corrective | C001/C002 closed; Eggress 1.0.8 qualified | `plans/subsystems/transport-probe-engine-corrective-addendum.md` |
| CLI/automation | closed corrective | C001 closed | `plans/subsystems/cli-automation-corrective-addendum.md` |
| Release/packaging | active | M004 closed (first standalone release qualified as 0.1.1); C001–C006 closed; M002 operationally qualified | `plans/subsystems/release-operational-qualification-roadmap.md` |
| Eggpack producer adoption | deferred | M003a blocked on selected closure-backed Eggpack producer interfaces | ADR-0002 + M003a |
| Eggup runtime self-update | deferred/optional | M003b blocked on manifest interop/consumer adapter + product decision | ADR-0002 + M003b |
| Native path/host diagnostics | active | M001/M002 closed; C002 upstream ICMP enablement ready; C003/M003 blocked; M004 ready; M005/M006 blocked | `plans/subsystems/native-path-host-diagnostics-roadmap.md` |
| QUIC/H3 expansion | roadmap-level | routed datagram composition remains separate from Phase 8 | Phase 9 prerequisites unresolved |

## Dependency-ready work

| Workstream | Milestone | Status | Plan | Handoff note |
|---|---|---|---|---|
| Release corrective | C005 first-release tag and hosted packaging qualification | closed | `plans/implementation/release-operational-qualification-corrective/005-first-release-tag-and-hosted-packaging-qualification.md` | Tag `v0.1.0` at `2760b8b`; hosted packaging run `36030784594` fully green; closure `plans/closure/release-operational-qualification-corrective/005-status.md` |
| Release corrective | C006 routed-path CryptoProvider installation | closed | `plans/implementation/release-operational-qualification-corrective/006-routed-path-cryptoprovider-installation.md` | Fix at `fa8ad7c`; hosted run `36038995837` green; closure `plans/closure/release-operational-qualification-corrective/006-status.md` |
| Native path/host diagnostics | M001 native contract/platform substrate | closed | `plans/implementation/native-path-host-diagnostics/001-native-contract-and-platform-substrate.md` | Schema 0.4 and `eggprobe-native` closed; `plans/closure/native-path-host-diagnostics/001-status.md` |
| Native path/host diagnostics | M002 target route/interface/egress MTU | closed | `plans/implementation/native-path-host-diagnostics/002-target-route-interface-and-egress-mtu.md` | Target-scoped route/interface/source/MTU evidence closed; `plans/closure/native-path-host-diagnostics/002-status.md` |
| Native path/host diagnostics corrective | C001 planning/documentation and execution realignment | closed | `plans/implementation/native-path-host-diagnostics-corrective/001-planning-documentation-and-execution-realignment.md` | Documentation/planning-only corrective; closure `plans/closure/native-path-host-diagnostics-corrective/001-status.md`. M004 now independently ready after C001; M003 continues as parallel blocked research line. |
| Native path/host diagnostics corrective | C002 ping-async upstream ICMP contract enablement | ready | `plans/implementation/native-path-host-diagnostics-corrective/002-ping-async-upstream-icmp-contract-enablement.md` | Requalify current upstream, land minimal additive payload/responder/structured-outcome interface; no Eggprobe git dependency |
| Native path/host diagnostics | M004 direct UDP service diagnostics | ready | `plans/implementation/native-path-host-diagnostics/004-direct-udp-service-diagnostics.md` | M001/M002 are closed; direct UDP is independent of M003 ICMP research after M001 (see native roadmap §6) |

## Release evidence gate

Release M004 closed with the first standalone archive release qualified as
`0.1.1` (tag `v0.1.1` at `53ea53d`, hosted packaging run `36041400748`;
closure `plans/closure/release-operational-qualification/004-status.md`).
The earlier stopped M004 execution at `25dc692` is retained as history in
the C006 plan and the M004 closure. The qualified `v0.1.1` artifact is the
immutable release of record; the `v0.1.0` artifact at `2760b8b` predates
the C006 production fix and remains immutable.

Release corrective C001's operational condition is **satisfied** by C005
(`plans/closure/release-operational-qualification-corrective/005-status.md`;
intentional tag `v0.1.0` at `2760b8b`, hosted packaging run `36030784594`,
inspected artifacts/hashes/native smoke). The C001 historical conditional
closure remains as evidence.

C004 closed with a fully green hosted matrix (run `36015806725` on
`04c84d3`; see `plans/closure/release-operational-qualification-corrective/004-status.md`).

Release M002 is **operationally qualified** (`plans/closure/release-operational-qualification/002-status.md` retained; its operational gate is met by C005 hosted evidence). None of these releases depends on Eggup or Eggpack adoption; the M003a/M003b Eggpack/Eggup integration paths remain blocked on their own readiness gates.

## Release identity hygiene

The qualified release of record is `v0.1.1` (tag at `53ea53d`); it
identifies the immutable archive published with the release M004 closure.
The current `main` branch carries unreleased Phase 8 native work
(Native M001/M002 at `6e45e85` plus the post-`v0.1.1` planning/diagnostic
changes recorded by this corrective).

Release/package workflows MUST NOT publish or recreate the changed
`main` tree under the `v0.1.1` tag. The next qualification cycle must
select a new version before artifact publication; the
`v0.1.0` (at `2760b8b`) and `v0.1.1` (at `53ea53d`) tags are immutable
and remain historical evidence.

## Registered blocked/deferred plans

| Workstream | Milestone | Status | Plan | Blocker / readiness gate |
|---|---|---|---|---|
| Eggpack producer adoption | M003a | blocked/deferred | `plans/implementation/release-operational-qualification/003a-eggpack-producer-release-integration.md` | Current Eggpack HEAD `154d4a2` still has Manifest M002, Build/Qualification M001, and Bootstrap M001 ready but not closed; selected interfaces must be closure-backed |
| Eggup runtime self-update | M003b | blocked/deferred | `plans/implementation/release-operational-qualification/003b-eggup-runtime-self-update-integration.md` | Current Eggpack HEAD `154d4a2` has interop M001 ready but not closed; Eggup HEAD `8937ad0` still gates adapter implementation on that closure; product decision also required |
| Release qualification | M004 | closed | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | First standalone release qualified as `0.1.1` (tag `v0.1.1` at `53ea53d`, hosted packaging run `36041400748`); closure `plans/closure/release-operational-qualification/004-status.md` |
| Native diagnostics corrective | C003 published ICMP backend adoption qualification | blocked | `plans/implementation/native-path-host-diagnostics-corrective/003-published-icmp-backend-adoption-qualification.md` | C002 closure plus a crates.io release containing the accepted upstream interface |
| Native diagnostics | M003 ICMP echo | blocked | `plans/implementation/native-path-host-diagnostics/003-icmp-echo-diagnostics.md` | C002 + C003 closure; consume only the published qualified backend, never a production Git pin |
| Native diagnostics | M005 traceroute/path | blocked | `plans/implementation/native-path-host-diagnostics/005-traceroute-path-diagnostics.md` | `tracert` 0.12.0 omits silent attempts, deduplicates responders, reverse-resolves by default, and adds a second `netdev` line |
| Native diagnostics | M006 active PMTU | blocked | `plans/implementation/native-path-host-diagnostics/006-active-path-mtu-discovery.md` | M004/M005 support seams remain open; trustworthy platform PMTU controls unqualified |

## Superseded planning

Historical Release M003 mixed producer release construction/mapping with
consumer deployment/update behavior.

- historical plan:
  `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md`;
- historical blocked disposition:
  `plans/closure/release-operational-qualification/003-status.md`;
- current disposition: **superseded by ADR-0002, M003a, and M003b**.

Do not execute the historical M003 plan and do not treat its old Eggup blocker
as a current first-release gate.

## Release ownership boundary

ADR-0002 is authoritative:

| Concern | Owner |
|---|---|
| target/artifact release identity | Eggpack |
| package/archive construction | Eggpack |
| final artifact manifest/digest/size evidence | Eggpack |
| bootstrap installer generation | Eggpack |
| generated release CI/staging/publication | Eggpack |
| release/version selection policy | Eggprobe |
| supported-target claims | Eggprobe |
| whether self-update is exposed | Eggprobe |
| consumer acquisition/verification | Eggup |
| local staging/locking/commit/rollback/recovery | Eggup |
| install receipt/service lifecycle | Eggup |

A standalone Eggprobe archive release may qualify before M003a or M003b.

## Current execution order

1. ~~Execute **Release corrective C005**~~ — done (`2760b8b`, tag `v0.1.0`,
   run `36030784594`): first-release workflow hygiene landed, one immutable
   first-release tag qualified, every artifact/checksum/identity record
   inspected, C001 evidence satisfied, M002 operationally qualified.
2. ~~Select the release candidate~~ — done as patch `0.1.1` (fix-forward;
   `v0.1.0` immutable and predates the C006 production fix).
3. ~~Execute **Release corrective C006**~~ — done (`fa8ad7c`, hosted run
   `36038995837`).
4. ~~Resume **Release M004** and qualify the first standalone archive
   release~~ — done: tag `v0.1.1` at `53ea53d`, hosted packaging run
   `36041400748` fully green, shipped-binary battery green, closure
   `plans/closure/release-operational-qualification/004-status.md`.
   Publication/distribution remains a separate decision (no GitHub
   Release exists).
5. Do not wait for M003a or M003b to complete the first release and do not
   create temporary Eggpack/Eggup substitutes inside Eggprobe.
6. M003a remains blocked until the specific Eggpack interfaces selected for
   adoption are closure-backed.
7. Author/execute M003b only if Eggprobe chooses to expose runtime self-update
   and the Eggpack->Eggup consumer seam is stable.
8. ~~Execute **Native M001**~~ — closed at `6e45e85`; hosted run `36051400630` green; closure `plans/closure/native-path-host-diagnostics/001-status.md`.
9. ~~Execute **Native M002**~~ — closed at `6e45e85`; hosted run `36051400630` green; closure `plans/closure/native-path-host-diagnostics/002-status.md`.
10. ~~Execute **Native corrective C001** before the next capability handoff~~ — closed; planning/docs/release-identity/M003↔M004 sequencing realigned; closure `plans/closure/native-path-host-diagnostics-corrective/001-status.md`.
11. **Native M004** may execute independently now.
12. Execute **Native corrective C002** in parallel: qualify/current-upstream `ping-async` and land the minimal additive payload/responder/structured-outcome contract.
13. After C002 closes, wait for an immutable crates.io release containing that exact interface, then execute **Native corrective C003**.
14. Execute **Native M003** only after C003 closes; consume the published qualified backend rather than a Git SHA/fork.
15. Keep M005 blocked for now; use C002 evidence to decide whether the same upstream ICMP surface can replace the rejected `tracert` path without losing silent-attempt/responder truth.
16. Execute M006 only after the route/UDP/path test seams it depends on are closure-backed and trustworthy PMTU feedback is available.
17. Keep routed datagram/QUIC work in Phase 9; do not reuse the byte-stream Eggress route contract for Phase 8.

## Shared infrastructure baselines

These are reviewed state, not permanent dependency pins:

| Project | Reviewed state | Relevant boundary |
|---|---|---|
| Eggfetch | 0.2.0 line | HTTP engine/custom dialer/origin TLS |
| Eggress | Eggprobe-qualified 1.0.8 line; upstream main also contains listener-free UDP work requiring separate adoption review | byte-stream routes remain current Eggprobe authority; routed datagrams are Phase 9 |
| Eggpack | Contract M002 + Manifest M001/M001a closed; Manifest M002/Build M001/Bootstrap M001/Eggup interop M001 ready at review | producer release authority |
| Eggup | producer distribution retired/transferred; core/acquisition/rollback consumer layers qualified; future Eggpack-manifest adapter not yet stable | consumer deployment authority |
| CodeGG planning convention | `239c51d19a63e8cf369a952b78d1e55092f4653b` | canonical -> ADR -> roadmap -> plan -> closure -> corrective lifecycle |

Implementation agents MUST re-check sibling interfaces at the actual execution
baseline.

## Historical evidence

| Work | Historical status | Evidence | Current note |
|---|---|---|---|
| Foundation M001/M002 + C001/C002 | closed | `plans/closure/foundation-diagnostic-contract/` and corrective closure | current contract qualified |
| Transport M001-M006 + C001/C002 | closed | transport closure trees | current transport qualified against Eggress 1.0.8 |
| CLI M001-M004 + C001 | closed | CLI closure trees | automation corrective closed |
| Release M001 | closed | `plans/closure/release-operational-qualification/001-status.md` | retained |
| Release M002 | operationally qualified | `plans/closure/release-operational-qualification/002-status.md` | historical conditional closure retained; operational gate met by C005 hosted evidence |
| historical Release M003 | blocked disposition | `plans/closure/release-operational-qualification/003-status.md` | superseded by ADR-0002 |

## Planning hygiene

- Historical closure/disposition records are preserved.
- Corrective addenda/ADRs are current authority where later evidence changes a
  planning assumption.
- Register a plan before implementation handoff.
- Do not promote blocked/deferred integration work before named sibling
  interfaces actually close.
- Release qualification uses demonstrated artifact/workflow evidence, not code
  inspection alone.
- Do not copy Eggfetch, Eggress, Eggpack, or Eggup ownership into Eggprobe to
  bypass an interface blocker.
- The registry is a compact active control surface. It points to closure and
  roadmap records rather than duplicating their narratives; release-identity
  notes stay short and link to the qualifying closure records.
- An immutable qualified release (`vX.Y.Z` tag at a fixed commit) is never
  rebuilt or re-published under the same tag after `main` changes. A new
  version is selected before the next qualification cycle.
