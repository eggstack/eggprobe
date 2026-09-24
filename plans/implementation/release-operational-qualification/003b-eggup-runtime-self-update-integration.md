# Release M003b — Optional Eggup Runtime Self-Update Integration

Status: blocked / deferred

Planning baseline: current planning head after ADR-0002 registration.

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`

Architecture:

- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`

Primary class: optional capability

## 1. Objective

Add an optional in-place Eggprobe self-update path using Eggup consumer
deployment machinery after the producer evidence -> consumer deployment seam is
stable.

This milestone is not required for ordinary archive installation or Release
M004 qualification unless Eggprobe chooses to advertise self-update.

## 2. Ownership boundary

Eggprobe owns:

- whether an update command is exposed;
- release/version selection policy;
- product-specific candidate identity requirements;
- user-facing opt-in/UX.

Eggpack owns:

- producer release manifest/evidence and artifact mapping.

Eggup owns:

- acquisition seam;
- integrity verification;
- candidate validation plumbing;
- staging/locking;
- commit/rollback/recovery;
- install receipt;
- optional service lifecycle primitives (not required by Eggprobe unless a
  future service mode exists).

## 3. Current blockers

Do not implement directly against the historical Eggup producer-distribution
surface; that subsystem is retired/transferred.

Readiness requires:

- a stable Eggpack final ReleaseManifest producer path relevant to Eggprobe;
- Eggpack-side interoperability contract/fixtures closed;
- a registered and stable Eggup-side manifest adapter or equivalent explicit
  translation seam;
- published or otherwise release-approved Eggup consumer crates/interfaces for
  the chosen integration;
- Eggprobe release/version-selection policy defined.

## 4. Intended flow

```text
Eggprobe release policy
        |
        v
authorized ProductId/ReleaseId/origin
        |
        v
Eggpack ReleaseManifest
        |
        v
Eggup manifest adapter / explicit translation
        |
        v
verified ArtifactSet + candidate validation
        |
        v
Eggup transaction
  stage -> verify -> commit -> rollback/recovery
```

The manifest is evidence, not release-selection authority.

## 5. Required behavior

- explicit current/target release identity;
- target/platform mismatch fails closed;
- size/digest verification before mutation;
- custom install location only through explicit supported policy;
- wrong candidate/version fails before commit;
- interruption before commit preserves current install;
- commit failure rolls back or returns typed RecoveryRequired;
- already-current is deterministic;
- no hidden privilege escalation;
- no unverified payload execution;
- no service-manager dependency unless separately justified.

## 6. CLI/API decision gate

Do not assume the command spelling from the historical M003 plan.

Before implementation, decide whether the product surface is:

- `eggprobe update`;
- a library/API integration only;
- or no self-update capability.

The decision should be based on the stable Eggup adapter and first-release
operational experience.

## 7. Tests

If implemented:

- local fake release origin;
- current/already-current/upgrade;
- wrong target;
- digest mismatch;
- wrong candidate identity/version;
- interrupted acquisition;
- pre-commit failure;
- commit rollback;
- RecoveryRequired path;
- custom destination if supported;
- redaction/no-secret logs.

No public release endpoint is required for correctness tests.

## 8. Stop conditions

Stop if implementing the milestone would require:

- restoring producer-side mapping into Eggup;
- importing Eggpack build/CI machinery into Eggup core;
- copying Eggup transaction logic into Eggprobe;
- making self-update a prerequisite for ordinary archive release.

## 9. Closure evidence

When eventually executed, create
`plans/closure/release-operational-qualification/003b-status.md` with exact
Eggpack/Eggup dependency versions, policy decision, local update matrix,
rollback/recovery evidence, and ownership review.
