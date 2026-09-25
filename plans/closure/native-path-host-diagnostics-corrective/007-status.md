# Native Corrective C007 Closure — Trippy #1793 Windows Fix Qualification and Upstream Handoff

Status: closed (Case C — distinct remaining defect, new upstream report filed)

Source plan: `plans/implementation/native-path-host-diagnostics-corrective/007-trippy-1793-windows-fix-qualification-and-upstream-handoff.md`

Source roadmap: `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Planning baseline: `f2167ee9332d94bf71c06babc183d87b059cc310`

Implementation: `f2167ee..30e9043` across three commits:

- `9195f89` — isolated reproducer harness (three detached variants with
  committed lockfiles), child-process evidence controller, isolated
  `workflow_dispatch` qualification job;
- `83e0047` — repro aligned exactly to the C005 qualifying smoke shape
  (3 rounds, `max_ttl` 3, default round floor, 10 s round budget) after the
  narrower 1-round probe hung without demonstrating the abort; controller
  hardened to stream per-run logs and classify timeouts;
- `30e9043` — archived the 26 per-run Case-C logs in-repo; added a
  `variants` dispatch input for single-variant re-qualification.

## Conclusion

**Case C: the C005 failure is not explained by upstream #1793.** The exact
#1793 fix commit aborts 10/10 and current master aborts 10/10 with the
byte-identical defining signature. C007 therefore closes as a successful
distinct-defect qualification and durable upstream handoff — not as a fix
qualification. No duplicate of #1793 was filed before this proof existed;
the new report below was filed only after the proof.

## Upstream state reviewed for this closure

- Issue #1793: `https://github.com/fujiapple852/trippy/issues/1793`
  (closed, milestone `0.14.0`; same panic class and `0xC0000409` abort as
  C005, diagnosed as overlapped `WSARecvFrom` `lpFlags` lifetime defect).
- Fix commit: `0b3c85bae2257915583d71392ea6d10c1cb4b75b`
  (`recv_flags: u32` stored in `SocketImpl`; verified present on master).
- Tested master: `c0c758eb1069151eafa6bd8227bb446c2b4d1506` (exactly the
  reviewed SHA from the plan; no advance occurred before the evidence run).
- Latest published release at evidence time: `0.13.0` (2025-05-05; no newer
  GitHub release). Fix milestone `0.14.0` still unreleased.
- Master-vs-fix diff for the receive path is non-behavioral (windows-sys
  `HANDLE`/`WSAEVENT` typing migration plus `run_with` `Action` plumbing),
  and master's `run_with` stays source-compatible via `R: Into<Action>`,
  so all three variants ran the identical reproducer source.
- New distinct-defect report filed by this closure: **`https://github.com/fujiapple852/trippy/issues/1881`**
  (`Windows: privileged UDP trace still aborts with 0xC0000409 after #1793`),
  with the standalone reproducer, bounded backtrace, environment, all three
  SHAs, run links, and the #1793 relationship analysis.

## Reproducer (durable, isolated)

`tools/repros/trippy-windows-1793/` (see its `README.md`):

- `baseline/` — crates.io `trippy-core =0.13.0`, `trippy-privilege =0.13.0`;
- `candidate-fix/` — git `rev 0b3c85b` for both crates (evidence-only);
- `candidate-master/` — git `rev c0c758e` for both crates (evidence-only);
- `run-evidence.ps1` — builds once, runs the binary as a child process per
  repeat (abort cannot kill the controller), streams per-run stdout/stderr
  to `logs/`, classifies abort/clean/other/timeout, asserts the expected
  family;
- each variant manifest carries its own empty `[workspace]` table, the
  Eggprobe root lists them under `exclude`, each variant commits its own
  `Cargo.lock`, and no variant depends on any `eggprobe-*` crate.

Trace configuration mirrors the C005 qualifying smoke exactly
(`trace 127.0.0.1 --max-hops 3`, default 3 attempts, engine ~10 s
per-round budget): `Protocol::Udp`, `PrivilegeMode::Privileged`,
`drop_privileges(false)`, `MultipathStrategy::Classic`,
`PortDirection::FixedSrc(43534)`, `first_ttl(1)`, `max_ttl(3)`,
`max_rounds(Some(3))`, `read_timeout(100ms)`, `max_round_duration(10s)`,
all other builder fields at trippy defaults. Loopback target only; no
public Internet target. Must run elevated; non-elevated runs exit `2`
without tracing.

## Windows evidence execution

Isolated manually-triggered job
`.github/workflows/trippy-1793-evidence.yml` (`workflow_dispatch`; never in
the ordinary `ci.yml` matrix):

- Run `36166315873` (`all` variants, on `83e0047`):
  `https://github.com/eggstack/eggprobe/actions/runs/36166315873`
- Run `36167102544` (`variants=candidate-master`, on `30e9043`):
  `https://github.com/eggstack/eggprobe/actions/runs/36167102544`
- Job name in both runs: `Elevated Windows baseline/candidate qualification`.

Exact environment (recorded by the jobs, identical across runs):

- Runner: Windows Server 2025 Datacenter, image `win25-vs2026`, X64.
- Elevation proof: `EVIDENCE elevated=True`, user `runneradmin`
  (plus `whoami /groups` high-integrity SID check in the workflow); every
  repro child independently reported `REPRO has_privileges=true`.
- Toolchain: `rustc 1.89.0 (29483883e 2025-08-04)` /
  `cargo 1.89.0 (c24e10642 2025-06-23)` on `x86_64-pc-windows-msvc`
  (the repo `rust-toolchain.toml` pin; also the product MSRV, which keeps
  this evidence directly relevant to C006 adoption review).
- `RUST_BACKTRACE=1`, debug profile.

Repeat counts and results:

| Variant | Dependency identity (from committed `Cargo.lock` + backtrace paths) | Runs | Result |
|---|---|---|---|
| baseline | registry `trippy-core 0.13.0` (`checksum dddf03…`; frames under `registry/…/trippy-core-0.13.0/`) | 3 | **3/3 abort**, defining signature |
| candidate-fix | git `rev 0b3c85b` (`trippy-core 0.14.0-dev`; frames under `git/checkouts/trippy-*/0b3c85b/`) | 10 | **10/10 abort**, identical signature |
| candidate-master | git `rev c0c758e` (`trippy-core 0.14.0-dev`; frames under `git/checkouts/trippy-*/c0c758e/`) | 10 | **10/10 abort**, identical signature |

Baseline abort (all 23 aborting runs share this shape):

- Exit code `-1073740791` (`0xC0000409 / STATUS_STACK_BUFFER_OVERRUN`).
- Stderr: `Layout::from_size_align_unchecked requires that align is a
  power of 2 …` followed by `thread caused non-unwinding panic. aborting.`
- Bounded backtrace excerpt: `Vec<u8>` corrupted-header drop inside
  `trippy_core::probe::UnknownExtension` → `Extension` → `Extensions` →
  `ProbeComplete` → `ProbeStatus` → `TracerState`, unwound from
  `Strategy::run` via `TracerInner::run_internal`/`run_with` — the same
  drop site and teardown path as the C005 forensics.
- Stdout markers prove each child reached the backend elevated before
  aborting (`REPRO has_privileges=true`, `REPRO tracer_built`; empty of any
  `RESULT` line).

Full per-run logs are archived in-repo (durable past artifact expiry and
branch deletion):

- `tools/repros/trippy-windows-1793/evidence/36166315873/` (26 logs:
  baseline ×3, candidate-fix ×10, plus run README);
- `tools/repros/trippy-windows-1793/evidence/36167102544/` (20 logs:
  candidate-master ×10, plus run README).

A narrower 1-round/2-TTL probe shape was tried first (run `36165388401`)
and hung without demonstrating the abort; it was replaced by the exact
C005 smoke shape above, which reproduces deterministically (3/3 then
10/10/10). The superseded run is retained in history but carries no
qualification weight.

## Why this is distinct from #1793 (not a regression)

- The #1793 `recv_flags` fix is present in both tested git trees (verified
  by patch inspection and by its persistence on master).
- The abort signature, drop site, and teardown path are unchanged to the
  frame, across 20/20 candidate runs on two different git SHAs.
- Therefore the `lpFlags` lifetime was at most one instance of Windows
  receive-path heap corruption; the Eggprobe reproducer's corruption source
  survives it. Upstream report `1881` carries this analysis.

## Production invariance confirmation

- `crates/eggprobe-native/Cargo.toml` still pins `trippy-core =0.13.0` /
  `trippy-privilege =0.13.0`; root `Cargo.lock` has no production
  dependency change (only a workspace `exclude` stanza for the harness;
  `cargo tree` still shows a single `trippy-privilege 0.13.0` line).
- The C005 Windows `Unsupported/HopProbe` refusal (`current_privilege_mode`)
  is unchanged and unweakened; no Windows trace execution was enabled.
- No Git SHA, fork, or vendored Trippy patch entered production manifests.
- Schema 0.4 untouched; regeneration produces no diff (below).

## Verification (closure head)

```text
cargo fmt --all -- --check                         PASS
cargo check --workspace --all-targets --all-features --locked PASS
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings PASS
cargo test --workspace --all-features --locked     PASS (98 passed, 0 failed)
cargo +1.89.0 check --workspace --all-targets --locked PASS
cargo tree --locked                                PASS (single trippy-privilege 0.13.0; trippy-core 0.13.0 only)
cargo +stable audit                                PASS (one pre-existing allowed unmaintained `paste` advisory, as in C005)
schema regeneration                                PASS (no diff)
```

Ordinary hosted CI (`ci.yml`) is unaffected: the evidence job is a separate
`workflow_dispatch` file and the harness crates are excluded from
`--workspace` verification (`cargo metadata` lists only the three
`eggprobe-*` members).

## Downstream disposition

- **C006** remains **blocked**, re-gated: entry now requires C007 closure
  (this record) plus a published immutable `trippy-core`/`trippy-privilege`
  release containing the fix for **upstream issue #1881** (or an explicitly
  reconciled equivalent). The #1793 fix alone is proven insufficient, so
  C006 must not accept a release merely because it contains `0b3c85b`.
  Production Git pins/forks/vendoring remain not accepted.
- **M005** Windows disposition is unchanged: live execution still blocked on
  the upstream backend fix (now tracked by #1881 via C006); the fail-safe
  refusal stays qualified.
- **M006** remains blocked independently on the PMTU DF/PTB control gap;
  untouched by C007.
- **M003/C003** remain blocked on upstream ICMP acceptance/merge/publication;
  untouched by C007.
- No future plan is unblocked by this closure: the correct next state is
  upstream action on #1881, then C006 re-qualification after publication.
