# CLI and Automation Roadmap

Status: closed through M004

Long-term references:

- `plans/000-long-term-specification.md#9-error-status-and-assertion-semantics`
- `plans/000-long-term-specification.md#10-serialization-and-automation`
- `plans/002-long-term-roadmap.md#phase-4--composite-diagnosis-assertions-and-comparison`
- `plans/002-long-term-roadmap.md#phase-5--plan-files-ndjson-and-batch-automation`

Related ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`

## 1. Purpose and ownership boundary

This subsystem owns CLI parsing, renderer selection, plan-file ingestion, assertion UX, batch/NDJSON surfaces, exit-code mapping, and automation ergonomics.

It consumes `eggprobe-core` typed APIs and MUST NOT own network semantics.

## 2. Work classification

### Invariants

- CLI args compile into canonical plan types before execution.
- JSON/NDJSON stdout contains machine data only.
- Human output is derived entirely from report data.
- Exit codes are a coarse projection of structured report state.
- CLI flags do not silently alter route/probe semantics beyond the plan they produce.

### Capabilities

- primitive subcommands;
- `check`;
- assertions;
- `compare`;
- JSON plan execution;
- NDJSON/batch mode;
- repeated probes and summary statistics;
- shell completions.

### Infrastructure

- Clap command model;
- renderer trait/modules;
- stdin/file loader;
- batch scheduler with bounded concurrency.

### Polish

- terminal tables;
- concise/default verbosity;
- color and no-color behavior;
- completions/manpage.

## 3. Non-goals

- no TUI;
- no daemon/server mode;
- no hidden config database;
- no shell pipeline execution;
- no interactive secret store.

## 4. Current state

No production CLI exists at planning bootstrap.

The foundation subsystem will create only a minimal CLI smoke boundary. Full command semantics belong here after probe capabilities exist.

## 5. Target architecture

```text
argv / stdin plan
      |
      v
CLI parser / plan loader
      |
      v
ProbePlan
      |
      v
eggprobe-core execute
      |
      v
ProbeReport
      |
      +--> HumanRenderer -> stdout
      +--> JsonRenderer  -> stdout
      `--> NdjsonEmitter -> stdout

logs/tracing/progress ----------------> stderr
```

## 6. Dependency graph

```text
foundation M001/M002
       |
transport M001-M005
       |
       v
CLI M001 primitive surface
       |
       v
CLI M002 check/assertions/exit codes
       |
       +--> CLI M004 compare/repetition
       |
       v
CLI M003 plan files/NDJSON/batch
```

M001 may begin once the primitive core probes it exposes are closed. It SHOULD not invent placeholder command semantics for absent probes.

## 7. Milestones

### M001 — Primitive command surface and renderers

Class: capability.

Target command families:

```text
eggprobe dns <host>
eggprobe tcp <host:port>
eggprobe tls <host:port>
eggprobe http <url>
eggprobe proxy <host:port> --via <route>
```

Common output selectors:

```text
--json
--ndjson   # only where streaming/repetition semantics are defined
--quiet
```

Exit conditions:

- each command maps to canonical plan types;
- human and JSON contain the same facts;
- machine stdout purity tests pass.

### M002 — Composite check, assertions, and exit codes

Class: capability.

Candidate assertions:

- status range;
- required HTTP version;
- required ALPN;
- required TLS version;
- maximum total/TTFB/connect duration;
- certificate-expiry threshold.

Exit conditions:

- probe failure versus assertion failure is explicit;
- exit-code mapping matches canonical spec;
- `check` preserves child evidence.

### M003 — Plan files, schema validation, NDJSON, and batch execution

Class: capability + infrastructure.

Exit conditions:

- stdin/file JSON plan equals CLI semantics;
- bounded batch concurrency;
- NDJSON record contract documented;
- generated schema usable by external validators;
- partial batch failures do not corrupt subsequent records.

### M004 — Direct-versus-route comparison and repeated statistics

Class: capability + polish.

Exit conditions:

- source reports preserved;
- repeated attempts remain individually addressable;
- aggregates declare sample count and statistic;
- no invalid comparison is manufactured when one side lacks evidence.

## 8. Cross-cutting requirements

### Compatibility

CLI spelling may evolve pre-1, but JSON semantics remain canonical. Flag aliases must not introduce alternate meanings.

### Security

Credential-bearing CLI arguments may be visible to local process inspection; documentation SHOULD prefer environment/file/secure-input forms where route credentials are needed. Regardless, reports/logs must redact them.

### Cancellation

Ctrl+C should cancel active work and exit 130 after cleanup.

### Performance

Batch mode enforces explicit concurrency bounds.

### Documentation

Every command documents human and machine output behavior plus examples.

## 9. Verification strategy

- snapshot/golden tests for human rendering;
- JSON parse tests rather than string contains;
- stdout/stderr separation tests;
- CLI-to-plan equivalence tests;
- signal/cancellation tests where portable;
- exit-code integration tests;
- batch ordering/partial-failure tests.

## 10. Risks and decision points

- NDJSON may represent per-probe events, per-plan final reports, or both; M003 must choose and document one stable record envelope before public release.
- Credential input UX may need a later secret-source abstraction.
- Color/table dependencies should remain optional/minimal.

## 11. Completion definition

The subsystem closes when interactive and automation users invoke the same engine, JSON is never a second-class formatting mode, batch behavior is bounded, and exit/finding semantics are stable and tested.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M001 primitive command surface and renderers | closed | `plans/implementation/cli-automation/001-primitive-command-surface-and-renderers.md` | `plans/closure/cli-automation/001-status.md` | — |
| M002 composite check/assertions/exit codes | closed | `plans/implementation/cli-automation/002-composite-check-assertions-and-exit-codes.md` | `plans/closure/cli-automation/002-status.md` | — |
| M003 plan files/schema/NDJSON/batch | closed | `plans/implementation/cli-automation/003-plan-files-schema-ndjson-and-batch.md` | `plans/closure/cli-automation/003-status.md` | — |
| M004 compare/repetition statistics | closed | `plans/implementation/cli-automation/004-route-comparison-and-repetition-statistics.md` | `plans/closure/cli-automation/004-status.md` | — |
