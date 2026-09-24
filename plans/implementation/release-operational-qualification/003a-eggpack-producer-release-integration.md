# Release M003a — Eggpack Producer Release Integration

Status: blocked / deferred

Planning baseline: current planning head after ADR-0002 registration.

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`

Architecture:

- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

Primary class: infrastructure + maintenance

## 1. Objective

Replace Eggprobe-owned producer release boilerplate with stable Eggpack
producer interfaces once those interfaces can represent and qualify Eggprobe's
existing release behavior without weakening evidence.

This milestone is not required for the first standalone Eggprobe archive
release.

## 2. Ownership boundary

Eggpack owns:

- target/artifact identity;
- release planning/build/qualification representation;
- final artifact aggregation;
- ReleaseManifest production;
- bootstrap installer generation;
- generated release CI/staging/publication where adopted.

Eggprobe owns only product-specific release policy/configuration and
qualification claims.

Eggup is not a dependency of this milestone.

## 3. Current blockers

At the reviewed Eggpack state:

- Contract M002 is closed;
- ReleaseManifest M001/M001a are closed;
- Manifest M002 final-artifact builder is ready but not yet closed;
- Build/Qualification M001 is ready but downstream build/qualification/
  finalization milestones are not closed;
- Bootstrap Installers M001 is ready but not closed;
- CI/release orchestration remains blocked on the build-plan interface.

Therefore an Eggprobe implementation handoff would currently bind to
unfinished producer interfaces.

## 4. Readiness gate

Promote this plan from blocked/deferred only when Eggpack has stable,
closure-backed interfaces sufficient for the exact Eggprobe adoption scope.

Minimum for manifest-only adoption:

- Eggpack Manifest M002 closed;
- a stable consumer-facing manifest/schema crate or CLI contract.

Minimum for replacing Eggprobe packaging/CI:

- relevant Eggpack build/qualification/finalization milestones closed;
- required bootstrap installer milestone closed if installers are adopted;
- generated CI/release orchestration interface closed if workflow generation is
  adopted.

Do not require every future Eggpack capability if a narrower adoption is
useful.

## 5. Intended work after readiness

- express Eggprobe release targets/artifact identity in Eggpack configuration;
- preserve current target set and qualification classification;
- generate/consume final Eggpack release manifest;
- remove duplicated target/archive/checksum mapping only after parity is
  demonstrated;
- optionally replace hand-maintained release CI/bootstrap with Eggpack-owned
  generated equivalents;
- retain release tag/source/version authority and machine-output smoke evidence;
- document migration/fallback.

## 6. Required parity matrix

At adoption time, prove:

- Linux x86_64;
- Linux aarch64 build qualification plus separate native SBC claim policy;
- macOS x86_64/arm64;
- Windows x86_64;
- archive naming/layout;
- final SHA-256;
- source/tag/package-version identity;
- `eggprobe --version`;
- schema-current JSON/NDJSON smoke;
- no automatic publication from mere build qualification.

## 7. Stop conditions

Stop rather than partially adopt if Eggpack cannot yet represent:

- final artifact identity without duplicate local mapping;
- build-vs-runtime qualification distinction;
- exact source/tag authority;
- final-byte digest evidence;
- the supported target matrix.

Do not add adapter glue that effectively recreates Eggpack logic inside
Eggprobe.

## 8. Closure evidence

When eventually executed, create
`plans/closure/release-operational-qualification/003a-status.md` with exact
Eggpack versions/commits, parity matrix, removed local duplication, generated
artifact evidence, and residual standalone fallback.
