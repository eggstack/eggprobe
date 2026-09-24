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

`54cbebbd50c269ee8b6271c45c5d124784ac54a9`

C003 corrective evidence baseline (planning head):

`cf857fc24579263611396d4d88a1ef622601d612`

GitHub Actions run `36003160519` at that head exposed the two release
qualification defects C003 was registered to repair: Windows
deterministic-fixture tests failed because checked-out JSON fixtures used
CRLF, and the audit job failed while bootstrapping `cargo-audit` under Rust
1.89. C003 closed those defects at their root; see the closure record for
the implementation commit and hosted CI evidence.

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
| Release/packaging | active | C003 closed; C001 conditionally closed; C002 closed | `plans/subsystems/release-operational-qualification-roadmap.md` |
| Eggpack producer adoption | deferred | M003a blocked on selected closure-backed Eggpack producer interfaces | ADR-0002 + M003a |
| Eggup runtime self-update | deferred/optional | M003b blocked on manifest interop/consumer adapter + product decision | ADR-0002 + M003b |
| Native path + QUIC/H3 expansion | roadmap-level | no implementation handoff | Phase 8/9 prerequisites unresolved |

## Dependency-ready work

| Workstream | Milestone | Status | Plan | Handoff note |
|---|---|---|---|---|
| Release corrective | C001 valid-tag hosted packaging evidence | conditionally closed | `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md` | C003 is now closed; C001's remaining gate is an intentional valid release tag + successful hosted packaging run with artifact/hash/native-smoke evidence |

## Operational evidence gate

C003 closed the cross-platform CI barrier; the next gate is C001's
valid-tag hosted artifact evidence.

Release corrective C001 remains **conditionally closed**:

- plan: `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md`;
- evidence: `plans/closure/release-operational-qualification-corrective/001-status.md`;
- remaining condition after C003 closes: create/use an intentional valid
  release tag and obtain a successful hosted packaging run with
  artifact/hash/native-smoke evidence.

Neither gate depends on Eggup or Eggpack adoption.

## Registered blocked/deferred plans

| Workstream | Milestone | Status | Plan | Blocker / readiness gate |
|---|---|---|---|---|
| Eggpack producer adoption | M003a | blocked/deferred | `plans/implementation/release-operational-qualification/003a-eggpack-producer-release-integration.md` | Current Eggpack HEAD `154d4a2` still has Manifest M002, Build/Qualification M001, and Bootstrap M001 ready but not closed; selected interfaces must be closure-backed |
| Eggup runtime self-update | M003b | blocked/deferred | `plans/implementation/release-operational-qualification/003b-eggup-runtime-self-update-integration.md` | Current Eggpack HEAD `154d4a2` has interop M001 ready but not closed; Eggup HEAD `8937ad0` still gates adapter implementation on that closure; product decision also required |
| Release qualification | M004 | blocked | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | C001 valid-tag hosted artifact evidence (M002/C001); C002 and C003 are closed |

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

1. Choose an intentional first release identity/tag and complete the
   remaining **C001 hosted evidence**, promoting M002 to operationally
   qualified. (C003 closed the cross-platform CI barrier; the release
   correctness correctives C001/C002 plus infrastructure C003 are all closed.)
2. With C003, C001/M002, and C002 closed, execute **Release M004** and qualify
   the first standalone archive release.
3. Do not wait for M003a or M003b to complete the first release and do not
   create temporary Eggpack/Eggup substitutes inside Eggprobe.
4. M003a remains blocked until the specific Eggpack interfaces selected for
   adoption are closure-backed.
5. Author/execute M003b only if Eggprobe chooses to expose runtime self-update
   and the Eggpack->Eggup consumer seam is stable.
6. Keep Phase 8/9 work roadmap-level until their platform/datagram ownership
   prerequisites exist.

## Shared infrastructure baselines

These are reviewed state, not permanent dependency pins:

| Project | Reviewed state | Relevant boundary |
|---|---|---|
| Eggfetch | 0.2.0 line | HTTP engine/custom dialer/origin TLS |
| Eggress | 1.0.8 line | listener-free routes + typed detailed errors |
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
| Release M002 | historical conditional closure | `plans/closure/release-operational-qualification/002-status.md` | current operational gate is C001 hosted evidence |
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
