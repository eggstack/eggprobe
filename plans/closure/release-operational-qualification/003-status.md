# Release M003 disposition

Status: blocked

Source plan: `plans/implementation/release-operational-qualification/003-shared-installer-update-integration.md`
Source roadmap: `plans/subsystems/release-operational-qualification-roadmap.md`
Repository baseline reviewed: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Blocker

`cargo search eggup --limit 10` returns no published crate, and the repository
contains no stable shared installer/updater API. The required interface
(verified asset mapping, checksum/provenance validation, atomic replacement,
rollback, and unsupported-platform errors) therefore cannot be consumed.

Per the source plan, no downloader, checksum, platform-selection, or binary
replacement logic was copied into Eggprobe. Manual archive installation remains
the truthful release path.

## Stop decision

The next requested plan, Release M004, depends on Release M002 and can be
qualified without advertising an updater, but the user requested stopping when
the next plan cannot be unblocked. Work stops here for reassessment of the
missing shared interface.

