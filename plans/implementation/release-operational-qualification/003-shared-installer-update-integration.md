# Release and Operational Qualification M003 — Shared Installer and Update Integration

Status: superseded by ADR-0002 and split M003a/M003b

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; retained as historical pre-boundary planning.

Source roadmap:

- historical M003 in `plans/subsystems/release-operational-qualification-roadmap.md`

Superseding architecture and plans:

- `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`
- `plans/implementation/release-operational-qualification/003a-eggpack-producer-release-integration.md`
- `plans/implementation/release-operational-qualification/003b-eggup-runtime-self-update-integration.md`

Historical dependency model: Release M002 + a shared Eggup installer/updater API.

This dependency model is superseded. Producer release construction belongs to
Eggpack; optional runtime deployment/update belongs to Eggup.

Primary class: historical capability plan; superseded

## 1. Objective

Provide verified install/update behavior by consuming shared Eggstack machinery rather than copying downloader, checksum, platform-selection, and replacement logic into Eggprobe.

## 2. Required interface capabilities

The shared library/tool should support:

- repository/application identity;
- current/target version selection;
- platform/architecture asset mapping;
- verified checksum/provenance validation;
- custom install directory;
- pinned version;
- atomic replacement/rollback behavior;
- clear unsupported-platform errors.

Eggprobe should supply policy/configuration, not own a parallel updater engine.

## 3. CLI surface

Expected eventual commands/options may include `eggprobe update` and installer script integration. Exact spelling must follow shared Eggstack conventions once `eggup` is stable.

## 4. Tests

- fresh install;
- pinned version;
- custom directory;
- upgrade;
- already-current;
- checksum mismatch;
- interrupted download/replacement;
- unsupported target;
- rollback/recovery;
- no execution of unverified payload.

## 5. Acceptance criteria

No duplicate updater implementation exists in Eggprobe. Downloads are verified before replacement. Failure leaves a runnable prior binary or a clear recoverable state.

## 6. Stop conditions

Stop if `eggup` has no stable interface, requires service-management features Eggprobe does not need, or lacks verification/atomicity. Keep M003 blocked rather than copying sibling updater code.

## 7. Closure evidence

Create `plans/closure/release-operational-qualification/003-status.md` with exact shared dependency/version, install/update matrix, integrity failures, rollback evidence, and ownership review.


## 8. Supersession note

This plan combined producer-side release mapping/bootstrap concerns with
consumer-side verified replacement/rollback. ADR-0002 separates those owners.

Do not execute this plan. Preserve it and
`plans/closure/release-operational-qualification/003-status.md` as historical
evidence of the earlier blocked architecture.

Use M003a for future Eggpack producer integration and M003b for optional Eggup
runtime self-update.
