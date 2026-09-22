# Closure and Verification Records

This directory contains evidence-based completion records for Eggprobe milestones.

A milestone is not closed merely because code landed.

## Layout

```text
closure/<subsystem>/NNN-status.md
```

Use the same milestone number as the source implementation plan.

## Required closure structure

A closure record MUST contain:

- status: closed, conditionally closed, corrective pass required, or blocked;
- source implementation plan and roadmap;
- repository baseline reviewed;
- implementation commit(s);
- executive finding;
- requirement-to-evidence matrix;
- production implementation evidence;
- exact verification commands and outcomes;
- invariant review;
- failure/cancellation/resource review;
- compatibility/schema review;
- security/redaction review;
- documentation/operations review;
- unresolved findings with severity;
- roadmap disposition;
- registry updates.

Do not report unrun commands as passing.

A milestone MUST NOT be closed when only compilation/formatting was verified, machine-readable compatibility was changed without fixtures, a public capability exists only as infrastructure, or a known high-severity correctness/security defect remains.
