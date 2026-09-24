# ADR-0002: Release Producer/Consumer Ownership

Status: accepted

Date: 2026-09-23

Decision owners: project maintainers

Related specification/roadmap sections:

- `plans/000-long-term-specification.md#14-platforms-and-packaging`
- `plans/002-long-term-roadmap.md#phase-7--release-packaging-and-operational-qualification`
- `plans/subsystems/release-operational-qualification-roadmap.md`

Related sibling architecture:

- Eggpack ADR-0001: producer/consumer release boundary.
- Eggup ADR-0004: Eggpack producer / Eggup consumer boundary.

## Context

Eggprobe's original Release M003 was written before the Eggstack release
infrastructure boundary stabilized. It grouped producer-side release mapping,
bootstrap installation, runtime acquisition, verification, and local
replacement under one "shared installer/update" milestone and treated a
published Eggup interface as the primary blocker.

That is no longer the Eggstack architecture.

Current sibling ownership is explicit:

- **Eggpack** owns producer release contracts, release conformance, package and
  archive construction, final release manifests, bootstrap-installer
  generation, generated release CI, staging/publication, and producer
  provenance/evidence.
- **Eggup** owns consumer acquisition, integrity verification, candidate
  validation, local staging/locking/commit/rollback/recovery, install receipts,
  and service lifecycle.
- **Eggprobe** owns product release/update policy: which release is acceptable,
  whether self-update is exposed, product-specific validation, and when
  producer or consumer integrations are adopted.

Eggup's former producer-side `eggup-dist` authority has been retired after the
contract/conformance behavior was moved to Eggpack.

Eggprobe already has a corrected standalone archive workflow. A first Eggprobe
release therefore does not need to wait for either Eggpack producer adoption or
an Eggup runtime updater.

## Decision drivers

- keep one producer authority across Eggstack;
- keep one consumer deployment/rollback authority;
- avoid duplicating target/asset/checksum mapping in Eggprobe;
- avoid making optional runtime self-update a prerequisite for ordinary binary
  releases;
- preserve a usable standalone release path while Eggpack matures;
- make future integration dependencies explicit rather than labeling all
  release work "blocked on Eggup."

## Decision

Eggprobe SHALL separate producer release integration from runtime self-update.

### Producer-side ownership

Future shared producer integration SHALL use Eggpack.

Eggpack is the authority for:

- portable target/artifact identity;
- final release manifests and final byte digests/sizes;
- package/archive construction;
- bootstrap installer generation;
- generated release CI/staging/publication;
- producer qualification/provenance evidence.

Eggprobe may retain its current standalone release workflow until the relevant
Eggpack producer interfaces are closed and adopting them provides a net
maintenance/reliability improvement.

### Consumer-side ownership

Future in-place `eggprobe update` behavior, if exposed, SHALL use Eggup's
consumer deployment mechanisms rather than copying downloader/verification/
replacement/rollback logic into Eggprobe.

An Eggpack manifest may be translated into Eggup deployment inputs through the
optional cross-repo interoperability seam. The manifest does not itself decide
which release Eggprobe should install.

### Product policy ownership

Eggprobe owns:

- release/version selection policy;
- whether an update command exists;
- product-specific candidate identity checks;
- opt-in/out and UX;
- which supported targets it advertises based on actual qualification evidence.

### First-release gate

A normal Eggprobe release consisting of qualified archives/checksums and
operator installation instructions SHALL NOT be blocked on Eggpack adoption or
Eggup self-update.

Release M004 may close without an updater as long as documentation makes that
limitation explicit.

## Consequences

Positive:

- the stale "blocked on Eggup" release gate is removed;
- producer mapping/packaging remains centralized in Eggpack;
- rollback/local mutation remains centralized in Eggup;
- Eggprobe can ship ordinary archives before either optional integration is
  ready;
- future self-update can consume Eggpack evidence without importing Eggpack
  build/CI machinery into Eggup core.

Negative:

- Eggprobe temporarily retains a standalone release workflow until Eggpack's
  producer pipeline is mature enough to adopt;
- there are two future integration milestones rather than one broad M003.

Neutral/deferred:

- whether Eggprobe ever exposes `eggprobe update`;
- exact Eggpack CLI/library integration surface;
- exact Eggup Eggpack-manifest adapter crate/API;
- package-manager distribution.

## Migration

The historical plan
`plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md`
is superseded as an active implementation plan. Its blocked disposition record
is retained as historical evidence of the pre-boundary architecture.

Replacement roadmap work:

- M003a — Eggpack producer release integration;
- M003b — optional Eggup runtime self-update integration.

Neither blocks the first ordinary archive release.

## Verification

Planning is conformant when:

- the registry no longer calls Eggup the producer-release blocker;
- Release M004 depends on actual artifact/qualification evidence, not optional
  self-update;
- Eggpack integration work names producer interfaces only;
- Eggup integration work names consumer transaction/deployment interfaces only;
- no Eggprobe plan proposes copying target/asset mapping, installer generation,
  verified replacement, or rollback machinery.
