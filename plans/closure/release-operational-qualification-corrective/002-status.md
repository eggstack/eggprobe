# Release Corrective C002 Closure

Status: closed

Source implementation plan:

- `plans/implementation/release-operational-qualification-corrective/002-release-boundary-and-documentation-cleanup.md`

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md`

Repository baseline reviewed: `1ba37d724c9be656d39d88f50176aca8e99fc088`

Implementation commit: `6d40de73b29a1b2d290dfc0cea786d5e8509ddf6`

## Executive finding

C002 is complete. User/operator documentation now describes Eggprobe's
standalone archive/checksum installation path and separates future Eggpack
producer adoption from optional Eggup consumer self-update. The old M003
blocked record remains intact as historical evidence; its current disposition
is explained by ADR-0002 and the M003a/M003b plans. Release M004 remains gated
on tagged hosted artifact evidence, not on either optional integration.

## Requirement-to-evidence matrix

| Requirement | Evidence |
|---|---|
| Standalone archive release and manual verification are accurately described | `README.md`; `docs/operator.md` installation section |
| Eggpack owns future shared producer construction/bootstrap/CI; adoption waits for stable qualified interfaces | `docs/operator.md`; `plans/adrs/ADR-0002-release-producer-consumer-ownership.md`; current upstream readiness review below |
| Eggup self-update is optional consumer integration and not a first-release blocker | `README.md`; `docs/operator.md`; `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md` |
| No unverified automatic download/replacement path is recommended | `docs/operator.md` installation and update guidance |
| Historical M003 disposition is preserved and replacement plans are linked | `plans/closure/release-operational-qualification/003-status.md` unchanged; superseded plan links ADR-0002/M003a/M003b; C001 closure now labels its old downstream note as a historical snapshot |
| M004 dependencies are consistent | M004 plan, subsystem roadmap, and `plans/registry.md`: C002 is closed; M002/C001 hosted artifact evidence remains required; M003b is conditional only if self-update is advertised |

## Production and planning evidence

Files changed:

- `README.md`
- `docs/operator.md`
- `plans/closure/release-operational-qualification-corrective/001-status.md` (historical downstream note annotated with its current disposition)
- `plans/implementation/release-operational-qualification-corrective/002-release-boundary-and-documentation-cleanup.md`
- `plans/implementation/release-operational-qualification/004-release-qualification-and-operator-docs.md`
- `plans/registry.md`
- `plans/subsystems/release-operational-qualification-roadmap.md`

The workflow comments and generated/help surface were reviewed. No updater or
shared-installer claim was present there, and no generated CLI/help surface was
changed.

### Downstream readiness recheck

The sibling repositories were checked at their current default-branch heads:

- Eggpack: `154d4a2ebd6b2703c9f368c108ccd5e75b2cf388`. Its active registry
  lists Release Manifest M002, Build/Qualification M001, Bootstrap Installers
  M001, and Eggup Interoperability M001 as ready handoffs, not closed work.
- Eggup: `8937ad01b18268bfdf2f32d1e42856e00829cd7a`. Its registry and
  distribution roadmap keep manifest consumption behind a stable Eggpack
  ReleaseManifest/interop seam; it has no registered Eggpack adapter
  implementation ready to consume that seam.

Therefore the M003a minimum manifest-only gate is not met: Manifest M002 and a
stable consumer-facing final-manifest contract are not closure-backed. M003a
remains `blocked / deferred`. M003b also remains `blocked / deferred`: the
Eggpack-side interop handoff is not closed, no Eggup-side adapter is ready, and
Eggprobe has not made the required product decision to expose self-update.
These are the explicit readiness gates already stated by M003a/M003b, not new
or unforeseen blockers. Neither milestone blocks first-release M004.

Current dependency graph:

```text
M002 packaging + C001 hosted evidence [operational evidence pending]
                         + C002 documentation/boundary cleanup [CLOSED]
                                      |
                                      v
                         M004 standalone release qualification
                                      |
                                      v
                         first archive release

Eggpack selected producer interfaces [not closure-backed] --> M003a [BLOCKED]
Eggpack manifest + interop [not closed] --> Eggup adapter + product decision
                                             --> M003b [BLOCKED / OPTIONAL]
```

## Verification commands and outcomes

- `rtk git diff --check` — passed before implementation commit.
- `rtk rg -n -i "blocked on (the )?(unpublished )?(shared )?eggup|eggup.*(required|blocker).*first release|updater is not yet|eggup interface.*not yet published|C002 release-boundary/docs cleanup \\[READY\\]" README.md docs plans/registry.md plans/subsystems/release-operational-qualification-roadmap.md plans/implementation/release-operational-qualification plans/implementation/release-operational-qualification-corrective plans/closure/release-operational-qualification plans/closure/release-operational-qualification-corrective` — no stale current-facing blocker claims or ready status remain; the only match is the C002 acceptance-criteria phrase asking reviewers to search for stale wording.
- Workflow/help review: `.github/workflows/release.yml`, `.github/workflows/release-skeleton.yml`, and the operator guide reviewed; no help test was applicable because generated/help text was not changed.

## Invariant and impact review

- **Ownership:** producer contracts/construction/bootstrap/CI remain Eggpack-owned; consumer acquisition and local replacement remain Eggup-owned; Eggprobe owns product policy.
- **Failure/cancellation/resources:** no executable behavior changed.
- **Compatibility/schema:** no CLI, report, schema, archive, or workflow behavior changed.
- **Security/redaction:** operator guidance requires checksum verification and rejects unverified automatic replacement scripts; no security boundary or credential handling code changed.
- **Documentation/operations:** manual archive installation and the separate optional status of runtime self-update are stated plainly.

## Unresolved findings

No unresolved C002 findings. M003a/M003b remain blocked by their named
cross-repository readiness requirements; M004 remains blocked by valid-tag
hosted packaging/artifact evidence recorded under C001.

## Roadmap and registry disposition

The release roadmap, registry, M004 plan, and C002 plan now reflect C002 as
closed. M003a and M003b remain blocked/deferred. M004 is still blocked only by
the operational M002/C001 artifact-evidence gate; its optional M003b condition
applies only if self-update is advertised. Historical M003 evidence is
preserved and clearly marked as superseded by ADR-0002 and M003a/M003b.
