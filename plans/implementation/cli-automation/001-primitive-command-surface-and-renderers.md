# CLI and Automation M001 — Primitive Command Surface and Renderers

Status: closed

Planning baseline: `e55a21fe73915b7a38c2fa674807354539806c60`; refresh when required core probes close.

Source roadmap:

- `plans/subsystems/cli-automation-roadmap.md#m001--primitive-command-surface-and-renderers`

Hard dependencies:

- Transport M001 for DNS/TCP commands.
- Transport M002 for TLS command.
- Transport M003 for HTTP command.
- Transport M004 for explicit proxy/route command semantics.

Primary class: capability

## 1. Objective

Expose implemented primitive diagnostics through a thin Clap frontend and pure human/JSON renderers without creating CLI-specific network behavior.

## 2. Required command surface

Expected families, adjusted only to match the final canonical target model:

```text
eggprobe dns ...
eggprobe tcp ...
eggprobe tls ...
eggprobe http ...
eggprobe proxy ... --via ...
```

Common controls should compile into `ProbePlan`: timeout/deadline, route, target policy, output mode, and probe-specific safe options.

## 3. Renderer contract

- human output is derived exclusively from `ProbeReport`;
- `--json` emits exactly one JSON document;
- stdout contains selected data only;
- logs/tracing/warnings outside report data go to stderr;
- no ANSI/color escapes in JSON;
- no renderer initiates network I/O.

NDJSON is deferred to M003.

## 4. Work packages

A. Build typed CLI args -> canonical plan conversion.
B. Add command-specific validation/help.
C. Add JSON renderer using canonical serialization.
D. Expand human renderer without adding unavailable facts.
E. Add exit plumbing for usage/internal/network result states without final assertion semantics.
F. Document examples.

## 5. Tests

- CLI-to-plan equality for every command;
- help/version snapshots;
- JSON parses as `ProbeReport`;
- stdout/stderr separation;
- no secrets in route display;
- human/JSON fact parity;
- invalid combinations fail before execution;
- Ctrl+C cleanup uses core cancellation semantics where supported.

## 6. Acceptance criteria

The CLI contains no socket/client/proxy logic. Every command invokes the same core engine. Machine output is clean and redacted. Human presentation does not invent facts.

## 7. Stop conditions

Stop if a command requires adding a second target/error model, if a required core probe is not closed, or if output semantics conflict with Foundation M002 schema.

## 8. Closure evidence

Create `plans/closure/cli-automation/001-status.md` with command matrix, CLI-to-plan fixtures, stdout/stderr tests, renderer parity, and ownership/dependency review.
