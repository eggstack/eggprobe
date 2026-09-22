# Release and Operational Qualification M004 — Release Qualification and Operator Documentation

Status: blocked

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh after release artifacts exist.

Source roadmap:

- `plans/subsystems/release-operational-qualification-roadmap.md#m004--release-qualification-and-operator-documentation`

Hard dependency: Release M002 closed.
Conditional dependency: M003 only if installer/update is advertised in the release being qualified.

Primary class: invariant + polish

## 1. Objective

Perform a release-candidate qualification pass that reconciles code, schema, artifacts, documentation, security evidence, and supported-platform claims before the first broadly consumable release.

## 2. Required qualification

- run strict workspace verification on release commit;
- verify generated schema/fixtures against release binary;
- verify all archives/checksums;
- perform install/extract smoke on supported hosts;
- execute representative local DNS/TCP/TLS/HTTP/proxy commands that are implemented by that release;
- verify JSON/NDJSON machine parsing;
- verify redaction corpus;
- run audit/dependency review;
- check MSRV source build;
- confirm version/tag/help/docs match;
- document unsupported/deferred capabilities truthfully.

## 3. Operator documentation

Provide:

- installation/source-build instructions;
- supported target table;
- command quick start;
- JSON/schema usage;
- proxy credential handling guidance;
- timeout/cancellation behavior;
- privacy/security notes;
- troubleshooting/error taxonomy;
- update instructions only if M003 is closed.

## 4. Acceptance criteria

No documentation claim exceeds qualified behavior. Release artifacts and schemas correspond to one source commit/version. No unresolved medium-or-higher release finding remains.

## 5. Stop conditions

Stop release qualification on checksum/provenance mismatch, schema drift, secret leak, failing supported-platform smoke, security advisory without disposition, or documentation claiming an unclosed capability.

## 6. Closure evidence

Create `plans/closure/release-operational-qualification/004-status.md` with a requirement-to-evidence matrix, artifact hashes, platform results, schema/redaction tests, security review, docs reconciliation, and final release disposition.
