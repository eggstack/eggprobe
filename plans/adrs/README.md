# Architecture Decision Records

This directory contains durable Eggprobe architecture decisions that cross milestones or establish public compatibility contracts.

## Naming

```text
ADR-NNNN-short-title.md
```

Numbers are monotonically increasing and never reused.

## Status lifecycle

```text
proposed -> accepted -> deprecated or superseded
         `-> rejected
```

Accepted ADRs are historical records. Create a new ADR to supersede one.

## Required template

```markdown
# ADR-NNNN: Title

Status: proposed

Date: YYYY-MM-DD

Decision owners: project maintainers

Related specification sections:

- `plans/000-long-term-specification.md#...`
- `plans/001-terminology-and-domain-model.md#...`

Affected subsystem roadmaps:

- `plans/subsystems/...`

## Context

## Decision drivers

## Considered options

### Option A — ...

### Option B — ...

## Decision

## Consequences

### Positive

### Negative

### Neutral or deferred

## Compatibility and migration

## Security and reliability implications

## Verification

## Supersession

None.
```

Use an ADR when a decision establishes or changes transport ownership, public JSON compatibility, security policy, routing semantics, a durable external dependency, or another cross-subsystem contract.
