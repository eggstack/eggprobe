# Release and Operational Qualification Roadmap

Status: active; M001 closed, M002 operationally qualified, C001 operational evidence satisfied by C005, C002/C003 closed historically, C004/C005 closed, M004 ready; historical M003 superseded

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
- C004 is closed (`plans/closure/release-operational-qualification-corrective/004-status.md`):
  hosted run `36015806725` on `04c84d3` is fully green (Ubuntu, macOS,
  Windows, MSRV, audit). The Windows-only 503 write-and-drop fixture
  now drains request headers (bounded), flushes, and shuts down
  orderly; the refused-hop probe retries on fresh ports with a 6 s
  per-attempt dial budget covering the hosted runner's ~2 s delayed RST
  for closed loopback ports. No production, schema, CLI, or release
  behavior changed. C003 remains historical closure evidence for the
  defects it actually fixed.
- current `main` `ff789d4c02d112ffb8909ba90f198ceebce1b6ad` is fully
  green in hosted CI run `36016882249`, so first-release artifact
  qualification is now the active gate;
- C005 has closed and supplied C001's remaining operational evidence and the
  M002 operational promotion: tag `v0.1.0` at `2760b8b`, hosted packaging run
  `36030784594` fully green across all five targets, independently verified
  artifact checksums, and host-native smoke on all host-native targets (Linux
  aarch64 build-qualified only). Evidence:
  `plans/closure/release-operational-qualification-corrective/005-status.md`.
  The pre-tag hygiene also replaced the stale `macos-13` Intel runner with
  `macos-15-intel` and made the smoke plan file-based so the native Windows
  binary can read it.

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

Therefore the first Eggprobe release is next gated by M004 final
release-candidate qualification and operator-document reconciliation, now
ready after C005. It is not blocked by Eggup or Eggpack adoption.

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
M002 standalone packaging [OPERATIONALLY QUALIFIED via C005]
              |
              +--> C002 release-boundary/docs cleanup [CLOSED]
              |
              +--> C003 CRLF/audit corrective [CLOSED HISTORICALLY]
                              |
                              v
              C004 Windows HTTP fixture + hosted CI requalification [CLOSED]
                              |
                              v
              C005 first-release tag + hosted packaging qualification [CLOSED]
                               |
                    supplied C001 remaining evidence
                    promoted M002 operationally qualified
                               |
                               v
M004 release qualification/operator docs [READY]
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

Status: operationally qualified via C005 (tag `v0.1.0`, hosted run
`36030784594`; see
`plans/closure/release-operational-qualification-corrective/005-status.md`).
Historical conditional closure retained as implementation evidence.

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

Status: ready for handoff. C005 has closed with C001 hosted artifact
evidence and the M002 operational promotion; C004, C002, and the original
C003 findings are closed.

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
| M002 cross-platform binary packaging | operationally qualified | `plans/implementation/release-operational-qualification/002-cross-platform-binary-packaging.md` | historical conditional closure + C005 hosted evidence (`plans/closure/release-operational-qualification-corrective/005-status.md`: tag `v0.1.0`, run `36030784594`) | — |
| C001 packaging workflow correction/hosted evidence | conditionally closed; operational evidence satisfied by C005 | `plans/implementation/release-operational-qualification-corrective/001-packaging-workflow-correction-and-hosted-evidence.md` | `plans/closure/release-operational-qualification-corrective/001-status.md` (historical) + `plans/closure/release-operational-qualification-corrective/005-status.md` (hosted evidence) | — |
| C002 release boundary/documentation cleanup | closed | `plans/implementation/release-operational-qualification-corrective/002-release-boundary-and-documentation-cleanup.md` | `plans/closure/release-operational-qualification-corrective/002-status.md` | — |
| C003 CI portability and standalone release requalification | closed historically; follow-up finding | `plans/implementation/release-operational-qualification-corrective/003-ci-portability-and-standalone-release-requalification.md` | `plans/closure/release-operational-qualification-corrective/003-status.md` | post-closure Windows findings owned and closed by C004 |
| C004 Windows HTTP fixture and hosted CI requalification | closed | `plans/implementation/release-operational-qualification-corrective/004-windows-http-fixture-and-hosted-ci-requalification.md` | `plans/closure/release-operational-qualification-corrective/004-status.md` (hosted run `36015806725` fully green) | — |
| C005 first-release tag and hosted packaging qualification | closed | `plans/implementation/release-operational-qualification-corrective/005-first-release-tag-and-hosted-packaging-qualification.md` | `plans/closure/release-operational-qualification-corrective/005-status.md` (tag `v0.1.0` at `2760b8b`, hosted run `36030784594` fully green) | — |
| historical M003 mixed installer/update | superseded | `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md` | historical blocked disposition retained | superseded by ADR-0002 |
| M003a Eggpack producer integration | blocked/deferred | `plans/implementation/release-operational-qualification/003a-eggpack-producer-release-integration.md` | pending | selected Eggpack producer interfaces not yet closure-backed |
| M003b Eggup runtime self-update | blocked/deferred | `plans/implementation/release-operational-qualification/003b-eggup-runtime-self-update-integration.md` | pending | Eggpack manifest/interoperability + Eggup consumer adapter + product decision |
| M004 release qualification/operator docs | ready | `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` | pending | — (C005 closed; C001/M002 operational evidence supplied) |
