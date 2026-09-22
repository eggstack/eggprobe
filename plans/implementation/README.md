# Milestone Implementation Plans

This directory contains bounded plans handed directly to implementation agents.

## Layout

```text
implementation/<subsystem>/NNN-short-title.md
```

## Required plan structure

Each plan MUST include:

- title and status;
- repository baseline;
- source roadmap;
- long-term requirements;
- applicable ADRs;
- primary work class;
- objective;
- readiness/dependencies;
- current implementation evidence;
- invariants;
- in-scope and out-of-scope work;
- required production changes;
- ordered work packages;
- failure/cancellation/restart/contention semantics;
- compatibility and migration;
- required tests;
- exact verification commands or command families;
- documentation updates;
- acceptance criteria;
- stop conditions;
- closure evidence;
- handoff notes.

Implementation plans are operational and baseline-specific. Agents may adjust file-level mechanics after inspecting current code, but may not weaken canonical invariants or silently expand into another subsystem.

Corrective work receives a new plan that references the original closure evidence.
