# Trippy #1793 Windows reproducer (C007 evidence harness)

Isolated, non-workspace evidence harness for
`plans/implementation/native-path-host-diagnostics-corrective/007-trippy-1793-windows-fix-qualification-and-upstream-handoff.md`.

It answers one question: does the already-landed upstream Trippy #1793 fix
remove the elevated-Windows abort Eggprobe captured in C005?

- Upstream issue: `https://github.com/fujiapple852/trippy/issues/1793`
  (same `Layout::from_size_align_unchecked` panic class and
  `0xC0000409 / STATUS_STACK_BUFFER_OVERRUN` abort Eggprobe observed).
- Upstream diagnosis: overlapped `WSARecvFrom` kept an invalid `lpFlags`
  pointer (`&mut 0` stack temporary).
- Fix commit: `0b3c85bae2257915583d71392ea6d10c1cb4b75b`
  (stores `recv_flags: u32` in `SocketImpl`, passes
  `addr_of_mut!(self.recv_flags)`).
- Reviewed master at plan time: `c0c758eb1069151eafa6bd8227bb446c2b4d1506`.
- Latest published release at plan time: `0.13.0` (milestone for the fix is
  `0.14.0`, still unreleased).

## Isolation invariants (do not weaken)

- This harness is **outside the Eggprobe Cargo workspace**: every variant
  manifest carries its own empty `[workspace]` table, the Eggprobe root
  manifest lists these crates under `exclude`, and no variant depends on any
  `eggprobe-*` crate. Production `Cargo.toml` / `Cargo.lock` are untouched.
- Git dependencies appear **only** inside this evidence harness. They MUST
  NOT be copied into Eggprobe production manifests (C006 still requires an
  immutable crates.io release containing the qualified fix).
- The reproducer targets loopback only (`127.0.0.1`); no public Internet
  target is required or used.
- The reproducer MUST run in an **elevated Windows process** (privileged UDP
  mode). Non-elevated runs exit `2` without tracing.

## Variants

| Directory | Dependency source | Expected result on elevated Windows |
|---|---|---|
| `baseline/` | crates.io `trippy-core =0.13.0`, `trippy-privilege =0.13.0` | non-unwinding abort, exit `0xC0000409` (`-1073740791`), stderr `Layout::from_size_align_unchecked` family |
| `candidate-fix/` | exact fix commit `0b3c85b` (git `rev`) | no abort: structured completion or recoverable typed backend error |
| `candidate-master/` | reviewed master `c0c758e` (git `rev`; record the actually-tested SHA if it advances) | no abort: structured completion or recoverable typed backend error |

All three variants share the same trace configuration, which mirrors the
C005 production builder (`crates/eggprobe-native/src/lib.rs::trace_path`):
`Protocol::Udp`, `PrivilegeMode::Privileged`, `drop_privileges(false)`,
`MultipathStrategy::Classic`, `PortDirection::FixedSrc(43534)`,
`first_ttl(1)`, bounded `max_ttl`/`max_rounds`, `min_round_duration(ZERO)`.
Each run performs one bounded single-round loopback trace and prints only
minimal `REPRO ...` markers (variant, config, elevation, result).

Each variant commits its own `Cargo.lock` so the tested dependency identity
(registry version or git SHA) is durable and reviewable after any branch is
deleted. The `logs/` directory holds workflow-captured evidence logs.

## Running the evidence

Preferred: the isolated manually-triggered workflow
`.github/workflows/trippy-1793-evidence.yml` (`workflow_dispatch`), which
records the runner image/OS, `rustc -Vv`, elevation proof, runs
`run-evidence.ps1` for each variant (baseline up to 3 attempts to demonstrate
the abort; each candidate 10 consecutive bounded runs with zero aborts), and
uploads the logs as artifacts. The intentionally-crashing baseline never
enters the ordinary always-on quality matrix.

Manual equivalent on an elevated Windows host (PowerShell, stable toolchain):

```powershell
.\run-evidence.ps1 -Variant baseline -Expect Abort -Repeats 3
.\run-evidence.ps1 -Variant candidate-fix -Expect Clean -Repeats 10
.\run-evidence.ps1 -Variant candidate-master -Expect Clean -Repeats 10
```

## Interpreting results

- **Case A** — baseline aborts with the defining signature, both candidates
  run clean 10/10: #1793 is the Eggprobe fix; C006 is bound to
  `0b3c85b`-containing publication. No new upstream issue.
- **Case B** — fix commit clean, master aborts: upstream regression report
  referencing #1793.
- **Case C** — fix commit still aborts: distinct-defect upstream report
  (`Windows: privileged UDP trace still aborts with 0xC0000409 after #1793`).

Failure to reproduce the baseline abort is a stop condition: reassess, do not
weaken the C005 Windows refusal and do not enable Windows tracing.
