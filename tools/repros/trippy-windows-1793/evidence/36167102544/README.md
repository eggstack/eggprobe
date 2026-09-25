# Evidence logs — hosted run `36167102544`

Workflow: `C007 Trippy 1793 Windows evidence` (`workflow_dispatch`,
`variants=candidate-master`) on `30e9043`, dispatched 2026-09-25T17:26:10Z.

Run URL: `https://github.com/eggstack/eggprobe/actions/runs/36167102544`

Environment (recorded by the job): Windows Server 2025 Datacenter,
runner image `win25-vs2026`, X64, elevated (`runneradmin`),
`rustc 1.89.0` / `cargo 1.89.0`, `RUST_BACKTRACE=1`.

Outcome (C007 Case C, master leg):

- `candidate-master-*.log` (10 runs): reviewed upstream master `c0c758e`
  aborts 10/10 with the byte-identical defining signature — exit
  `-1073740791` (`0xC0000409`) and `Layout::from_size_align_unchecked`
  while dropping a parsed `UnknownExtension` in `Strategy::run` →
  `TracerState` teardown (backtrace frames resolve to
  `.cargo/git/checkouts/trippy-*/c0c758e/crates/trippy-core`, proving the
  tested dependency identity).

The baseline leg (registry 0.13.0, 3/3 aborts) and the exact-fix-commit leg
(`0b3c85b`, 10/10 aborts) are archived under `../36166315873/`. Stdout logs
hold the `REPRO ...` markers; stderr logs hold the panic text plus the
bounded backtrace. No Eggprobe production code ran here; these are
isolated-harness logs only.
