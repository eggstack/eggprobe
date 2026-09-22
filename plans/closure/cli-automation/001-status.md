# CLI M001 closure

Status: closed

Source plan: `plans/implementation/cli-automation/001-primitive-command-surface-and-renderers.md`
Implementation: `792ad65524a869d2ec22ba88dd9daea427c74cab`

## Finding and evidence

The Clap surface exposes `dns`, `tcp`, `tls`, `http`, and `proxy`; every command
constructs a canonical `ProbePlan` and invokes `ProbeEngine`. Human output is
derived from `ProbeReport`; JSON uses canonical Serde and contains no ANSI or
logs. Route output is opaque. CLI integration tests cover help/version and the
core renderer; local `dns localhost --json` produced one parseable report.

## Verification

16 workspace tests, strict Clippy, MSRV, locked checks, and audit passed.
No network/client logic exists in the CLI crate.

