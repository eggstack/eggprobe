# Release and Operational Qualification Roadmap

Status: active; M001 closed, M002 operational evidence pending, C002/C003 closed historically, C004 ready; historical M003 superseded

Long-term references:

- `plans/000-long-term-specification.md#14-platforms-and-packaging`
- `plans/002-long-term-roadmap.md#phase-7--release-packaging-and-operational-qualification`

Related ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

## 1. Purpose and ownership boundary

This subsystem owns Eggprobe-specific release policy, supported-target claims,
release qualification, operator documentation, and the adapters/configuration
by which Eggprobe may consume shared Eggstack release infrastructure.

It does not own:

- shared producer release construction, manifests, bootstrap generation, or
  generated release CI — those belong to Eggpack;
- generic verified local deployment, replacement, rollback, or recovery —
  those belong to Eggup;
- diagnostic semantics — those belong to the foundation/transport/CLI
  subsystems.

Eggprobe may retain a small standalone release workflow until the corresponding
Eggpack producer interfaces are mature enough to replace it safely.

## 2. Work classification

### Invariants

- release binaries correspond to tested source;
- checksums/provenance are published for advertised artifacts;
- MSRV claims are tested;
- unsupported targets are not advertised as supported;
- release smoke tests validate machine output as well as human CLI behavior;
- first-release qualification is not blocked on optional Eggpack/Eggup
  integrations;
- producer mapping/build/bootstrap/CI logic is not reimplemented locally once
  Eggpack is adopted;
- generic verified mutation/rollback logic is not copied from Eggup.

### Capabilities

- downloadable standalone binary;
- documented manual/archive installation;
- version command;
- shell completions;
- optional future bootstrap installer through Eggpack;
- optional future in-place self-update through Eggup.

### Infrastructure

- current standalone CI target matrix;
- current release packaging workflow;
- checksum/provenance generation;
- smoke fixtures;
- future Eggpack producer adapter/configuration;
- future Eggup consumer adapter integration.

### Polish

- compact binary profile;
- package-manager distribution;
- man pages.

## 3. Non-goals

- no auto-update daemon;
- no telemetry requirement;
- no package-manager matrix before direct release artifacts are stable;
- no local producer framework duplicating Eggpack;
- no local updater transaction engine duplicating Eggup;
- no requirement that optional self-update exist before the first archive release.

## 4. Current state

The diagnostic product correctives are closed through the current schema and
transport/CLI qualification.

Release state:

- M001 CI/MSRV/audit/release skeleton is historically closed;
- M002 packaging implementation exists;
- Release corrective C001 repaired cross-architecture smoke, checksum, and
  tag/source/version authority defects;
- C001 is conditionally closed because the repository has no existing release
  tag, so a successful hosted packaging run/artifact evidence does not yet
  exist;
- C002 release-boundary/documentation cleanup is closed;
- C003 closed the specific Windows CRLF fixture and audit-tool bootstrap
  findings. Canonical JSON fixtures and schemas are LF-locked via
  `.gitattributes`; the audit job uses an explicit stable toolchain, pinned
  `cargo-audit 0.22.2`, and `cargo +stable audit`.
- Post-C003 hosted run `36008924756` proved those two corrections work:
  Ubuntu, macOS, MSRV, and dependency audit are green, and the Windows
  deterministic contract tests now pass. The same Windows job later fails
  `http_status_is_observed_even_when_assertion_fails` in
  `crates/eggprobe-core/tests/engine.rs`, where a local fixture is expected
  to yield an observed HTTP 503 with probe status `Ok` but instead yields
  report status `Failed`.
- C004 is registered to diagnose and correct that Windows-only HTTP fixture /
  transport-lifecycle failure and to obtain the fully green hosted matrix that
  first-release qualification requires. C003 remains historical closure
  evidence for the defects it actually fixed.

Current shared-infrastructure state reviewed during the ownership correction:

### Eggpack

- Contract M002 is closed;
- ReleaseManifest M001 and corrective M001a are closed;
- Manifest M002 final-artifact builder is ready but not closed;
- Build/Qualification M001 is ready but later builder/qualification/finalizer
  milestones are not closed;
- Bootstrap Installers M001 is ready but not closed;
- CI/release orchestration is still downstream of the build-plan interface;
- Eggpack-side Eggup interoperability M001 is ready but not closed.

### Eggup

- producer-side distribution authority has been retired/transferred to Eggpack;
- `eggup-dist` is removed;
- consumer update transaction/acquisition/rollback layers are qualified;
- future Eggpack-manifest consumption remains optional and gated on a stable
  interoperability adapter.

Therefore the first Eggprobe release is blocked first by C004 current-head
Windows qualification and then by hosted artifact evidence (C001/M002), not
by Eggup and not by Eggpack adoption.

## 5. Target architecture

```text
                         first-release path
source/tag
   |
   v
Eggprobe standalone packaging workflow
   |
   v
qualified archives + checksums
   |
   v
M004 release qualification/docs
   |
   v
manual/archive installation
```

Future shared producer path:

```text
Eggprobe product release policy/config
                 |
                 v
              Eggpack
 contract -> build/qualify -> finalize -> manifest
                 |
                 +--> bootstrap installer / generated CI when adopted
```

Optional future runtime self-update path:

```text
Eggprobe release-selection policy
            |
            v
Eggpack ReleaseManifest evidence
            |
            v
Eggup consumer adapter/transaction
            |
            v
verified local install / rollback / receipt
```

The future paths are additive maintenance/capability work. They are not required
to qualify an ordinary standalone archive release.

## 6. Dependency graph

```text
M001 CI/MSRV/release skeleton [CLOSED]
              |
              v
M002 standalone packaging [IMPLEMENTED; OPERATIONAL EVIDENCE PENDING]
              |
              +--> C002 release-boundary/docs cleanup [CLOSED]
              |
              +--> C003 CRLF/audit corrective [CLOSED HISTORICALLY]
                              |
                              v
              C004 Windows HTTP fixture + hosted CI requalification [READY]
                              |
                              v
              C001 remaining hosted packaging evidence
                    [CONDITIONALLY CLOSED / HOLD]
                              |
                              v
M004 release qualification/operator docs
              |
              v
first standalone release

Optional/deferred:

Eggpack stable producer interfaces
              |
              v
M003a Eggpack producer release integration

Eggpack manifest/interoperability + Eggup consumer adapter
              |
              v
M003b optional Eggup runtime self-update
```

M003a and M003b are independent of first-release qualification unless the
release explicitly advertises those capabilities.

## 7. Milestones

### M001 — CI, MSRV, audit, and release skeleton

Class: infrastructure.

Exit conditions:

- format, strict Clippy, tests, audit;
- MSRV lane;
- primary platform compile/test matrix;
- release workflow skeleton does not publish unverified artifacts.

Status: closed.

### M002 — Cross-platform binary packaging

Class: capability + infrastructure.

Exit conditions:

- target archives and checksums;
- local/release smoke test;
- `eggprobe --version`;
- machine-output smoke parseable;
- Linux aarch64/SBC claims distinguish build qualification from native runtime
  qualification.

Status: implementation/corrective code exists; operationally pending C001 hosted
evidence against a valid tag.

### Historical M003 — Shared installer/update integration

Status: superseded.

The historical plan mixed producer and consumer responsibilities and is
retained only as planning history:

- `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md`
- `plans/closure/release-operational-qualification/003-status.md`

ADR-0002 replaces it with M003a and M003b.

### M003a — Eggpack producer release integration

Class: infrastructure + maintenance.

Objective:

Adopt stable Eggpack producer contracts/build/finalization/manifest/bootstrap/CI
surfaces where they reduce duplicated Eggprobe release machinery.

Implementation plan:

- `plans/implementation/release-operational-qualification/003a-eggpack-producer-release-integration.md`

Status: blocked/deferred on closure-backed Eggpack interfaces required by the
chosen adoption slice.

This milestone is not required for the first standalone release.

### M003b — Optional Eggup runtime self-update integration

Class: optional capability.

Objective:

If Eggprobe elects to provide in-place self-update, use Eggup consumer
deployment/rollback machinery with Eggpack release evidence rather than local
updater logic.

Implementation plan:

- `plans/implementation/release-operational-qualification/003b-eggup-runtime-self-update-integration.md`

Status: blocked/deferred on stable Eggpack manifest/interoperability plus an
Eggup-side consumer adapter/API and a product decision to expose self-update.

This milestone is not required for the first standalone release.

### M004 — Release qualification and operator documentation

Class: polish + invariant.

Objective:

Qualify one concrete standalone release candidate and reconcile artifacts,
schemas, security evidence, target claims, and operator documentation.

Exit conditions:

- installation/archive instructions;
- supported target table;
- release verification record;
- schema fixture compatibility against release binary;
- dependency/security evidence;
- no unresolved medium-or-higher release blocker;
- update instructions only if optional M003b is actually closed/advertised.

Status: blocked on C004 closure and then M002/C001 hosted artifact evidence
for the first standalone release; C002 and the original C003 findings are
closed.

## 8. Cross-cutting requirements

### Compatibility

Release version and schema version are separate.

### Security

No unsigned/unverified download path should be presented as the default
installer behavior. Security audit exceptions require documented rationale and
expiry/review.

Eggpack manifests provide producer evidence; they do not authorize a release
selection. Eggup mutation must verify the selected candidate before commit.

### Performance

Binary-size/throughput claims require measurement. Small release profiles may
be evaluated but must not compromise diagnostics or unwind safety.

### Operations

A source build remains available even when no prebuilt target exists. Manual
archive installation remains a supported truthful path until optional shared
integration replaces it.

## 9. Verification strategy

First-release qualification:

- fully green current-head CI across Linux, macOS, Windows, MSRV, and the
  dependency-audit lane before selecting/tagging the release candidate;
- Windows HTTP fixture behavior must distinguish a successfully observed HTTP
  error status from transport failure and must use portable socket lifecycle
  semantics;
- hosted artifact build against an existing release tag;
- release archive extraction;
- `--version`;
- machine-output parse smoke;
- representative local loopback diagnostics;
- checksum verification;
- MSRV compile;
- target-native tests where runners exist;
- artifact inspection.

Future Eggpack adoption additionally requires parity against the standalone
release behavior before deleting local release mapping/workflow code.

Future Eggup self-update requires deterministic local update/rollback fixtures;
public release endpoints are not correctness dependencies.

## 10. Risks and decision points

- Windows machine-contract fixtures must not depend on checkout newline
  conversion; canonical JSON fixtures need cross-platform byte semantics.
- Security-audit tooling must not accidentally inherit the product MSRV when
  the audit tool itself requires a newer compiler.
- Windows networking semantics may require platform-specific qualification.
- Linux aarch64 cross-builds are not equivalent to target-class runtime
  evidence; native SBC evidence is required before claiming native runtime
  qualification.
- Eggpack adoption too early could bind Eggprobe to unfinished producer APIs.
- Eggup self-update should not be implemented merely because the transaction
  layer exists; product UX/policy must justify it.
- temporary standalone release machinery should not grow into a second
  Eggpack.

## 11. Completion definition

For the first release, this subsystem may close when users can obtain, verify,
install manually, and run a supported binary and the release artifacts have
machine-contract smoke evidence.

Eggpack producer integration and Eggup runtime self-update are subsequent,
optional/deferred milestones and do not prevent that closure.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure/evidence | Blockers |
|---|---|---|---|---|
| M001 CI/MSRV/audit/release skeleton | closed | `plans/implementation/release-operational-qualification/001-ci-msrv-audit-release-skeleton.md` | `plans/closure/release-operational-qualification/001-status.md` | — |
| M002 cross-platform binary packaging | operational evidence pending | `plans/implementation/release-operational-qualification/002-cross-platform-binary-packaging.md` | historical conditional closure + corrective C001 | C004 closure, then valid release tag and hosted run evidence |
| C001 packaging workflow correction/hosted evidence | conditionally closed / hold | `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md` | `plans/closure/release-operational-qualification-corrective/001-status.md` | C004 closure, then valid release tag + hosted artifact evidence |
| C002 release boundary/documentation cleanup | closed | `plans/implementation/release-operational-qualification-corrective/002-release-boundary-and-documentation-cleanup.md` | `plans/closure/release-operational-qualification-corrective/002-status.md` | — |
| C003 CI portability and standalone release requalification | closed historically; follow-up finding | `plans/implementation/release-operational-qualification-corrective/003-ci-portability-and-standalone-release-requalification.md` | `plans/closure/release-operational-qualification-corrective/003-status.md` | post-closure Windows HTTP test finding is owned by C004 |
| C004 Windows HTTP fixture and hosted CI requalification | ready | `plans/implementation/release-operational-qualification-corrective/004-windows-http-fixture-and-hosted-ci-requalification.md` | pending | — |
| historical M003 mixed installer/update | superseded | `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md` | historical blocked disposition retained | superseded by ADR-0002 |
| M003a Eggpack producer integration | blocked/deferred | `plans/implementation/release-operational-qualification/003a-eggpack-producer-release-integration.md` | pending | selected Eggpack producer interfaces not yet closure-backed |
| M003b Eggup runtime self-update | blocked/deferred | `plans/implementation/release-operational-qualification/003b-eggup-runtime-self-update-integration.md` | pending | Eggpack manifest/interoperability + Eggup consumer adapter + product decision |
| M004 release qualification/operator docs | blocked | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | pending | C004 closure, then M002/C001 hosted evidence |
