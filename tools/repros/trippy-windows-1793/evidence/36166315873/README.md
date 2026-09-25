# Evidence logs — hosted run `36166315873`

Workflow: `C007 Trippy 1793 Windows evidence` (`workflow_dispatch`, `all`
variants) on `83e0047`, dispatched 2026-09-25T17:18:42Z.

Run URL: `https://github.com/eggstack/eggprobe/actions/runs/36166315873`

Environment (recorded by the job): Windows Server 2025 Datacenter,
runner image `win25-vs2026`, X64, elevated (`runneradmin`,
`EVIDENCE elevated=True`), `rustc 1.89.0` / `cargo 1.89.0`,
`RUST_BACKTRACE=1`.

Outcome (C007 Case C):

- `baseline-*.log` (3 runs): registry `trippy-core 0.13.0` aborts 3/3 with
  the defining signature — exit `-1073740791` (`0xC0000409`) and
  `Layout::from_size_align_unchecked` while dropping a parsed
  `UnknownExtension` in `Strategy::run` → `TracerState` teardown.
- `candidate-fix-*.log` (10 runs): exact upstream #1793 fix commit
  `0b3c85b` aborts 10/10 with the byte-identical signature (backtrace
  frames resolve to
  `.cargo/git/checkouts/trippy-*/0b3c85b/crates/trippy-core`, proving the
  tested dependency identity). The job correctly stopped before the master
  variant; master-only qualification ran separately (see the C007 closure).

Stdout logs hold the `REPRO ...` markers (variant, config, `has_privileges=true`,
`tracer_built`); stderr logs hold the panic text plus the bounded backtrace.
No Eggprobe production code ran here; these are isolated-harness logs only.
