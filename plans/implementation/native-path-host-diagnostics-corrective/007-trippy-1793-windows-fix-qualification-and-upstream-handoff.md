# Native Path and Host Diagnostics Corrective C007 — Trippy #1793 Windows Fix Qualification and Durable Upstream Handoff

Status: closed (Case C — distinct remaining defect; see closure)

Closure: `plans/closure/native-path-host-diagnostics-corrective/007-status.md`

Repository baseline: `f2167ee9332d94bf71c06babc183d87b059cc310`

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Corrective evidence:

- `plans/closure/native-path-host-diagnostics-corrective/005-status.md`
- `plans/implementation/native-path-host-diagnostics-corrective/006-windows-trace-requalification-on-fixed-backend.md`

Relevant hosted Eggprobe evidence:

- C005 qualifying run `36156303047` on `53226a3`
- Windows job `108141559394` — qualified fail-safe refusal
- C005 debug runs `36152587509` / `36153274781` — elevated-Windows abort/backtrace evidence

Upstream Trippy evidence reviewed for this plan:

- issue #1793: `https://github.com/fujiapple852/trippy/issues/1793`
- fix commit: `0b3c85bae2257915583d71392ea6d10c1cb4b75b`
- current master reviewed: `c0c758eb1069151eafa6bd8227bb446c2b4d1506`
- latest published release reviewed: `0.13.0` (no newer GitHub release as of 2026-09-25)
- #1793 milestone: `0.14.0`

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

Primary work class: upstream qualification + durable evidence handoff

## 1. Objective

Determine whether the already-landed upstream Trippy #1793 fix is the exact fix
for Eggprobe's C005 elevated-Windows tracer abort, preserve a minimal durable
reproducer/evidence package, and bind C006 to a verified upstream fix before
waiting for crates.io publication.

Do not file a duplicate Trippy issue merely because Eggprobe observed the same
crash. Upstream #1793 already reports the same defining signature:

```text
Layout::from_size_align_unchecked ...
process exit 0xc0000409 / STATUS_STACK_BUFFER_OVERRUN
```

Upstream root-caused #1793 to an overlapped `WSARecvFrom` lifetime bug:
`lpFlags` was passed as `&mut 0`, leaving Winsock with a pointer to a
stack-backed temporary after the call returned. Commit `0b3c85b` fixes this
by storing `recv_flags` in `SocketImpl` and passing
`addr_of_mut!(self.recv_flags)`.

C007 must prove whether that fix removes the Eggprobe reproducer's abort under
the same elevated-Windows/privileged-UDP conditions. Production Eggprobe
remains pinned to 0.13.0 and continues refusing the Windows backend until C006
qualifies a published release.

## 2. Why this work is separate from C006

C006 is a production dependency-adoption/requalification plan and correctly
refuses Git/fork/vendor production bridges.

C007 is evidence-only and may use exact upstream commits in an isolated
standalone reproducer to answer one question before publication:

> Does #1793 actually fix the same crash Eggprobe observed?

Keeping this separate prevents two bad outcomes:

1. waiting for a future release only to discover that #1793 was not the same
   defect; or
2. filing a duplicate upstream issue when the bug is already fixed on master
   but unreleased.

C007 does not change Eggprobe runtime behavior.

## 3. Current upstream state

As reviewed on 2026-09-25:

- Trippy `0.13.0` remains the latest published GitHub release;
- issue #1793 is closed and assigned to milestone `0.14.0`;
- #1793's reported Windows failure is the same panic class and process abort
  code captured by C005;
- maintainer diagnosis: overlapped `WSARecvFrom` retained an invalid pointer
  because `lpFlags` referenced `&mut 0`;
- fix commit `0b3c85b` adds a persistent `recv_flags: u32` field to
  `SocketImpl`;
- current master `c0c758e` contains that fix but is not an immutable
  production dependency release.

This is strong evidence of a match, but not yet Eggprobe-specific proof.

## 4. Invariants

C007 MUST preserve:

1. Eggprobe production `Cargo.toml` / `Cargo.lock` remain on the accepted
   published Trippy 0.13.0 line until C006 executes.
2. The C005 Windows `Unsupported/HopProbe` refusal remains unchanged.
3. No Git SHA, fork, or vendored Trippy patch enters production dependency
   resolution.
4. The repro is isolated from the Eggprobe workspace/product dependency graph.
5. The baseline and candidate use the same trace configuration, target,
   privilege mode, compiler family, and host conditions wherever possible.
6. The baseline must demonstrate the defining C005/#1793 failure signature
   before a candidate can be called a fix.
7. "Did not crash once" is insufficient; bounded repeated runs are required.
8. Candidate success means no process abort/heap-corruption signature. A normal
   structured trace completion or recoverable typed backend error is acceptable
   for upstream-safety qualification; C006 separately owns Eggprobe live
   capability qualification.
9. No public Internet target is required; loopback is the canonical repro.
10. Evidence must be durable and reviewable after temporary CI/debug branches
    are deleted.
11. If #1793 does not fix the Eggprobe repro, do not weaken the C005 refusal;
    file a distinct upstream report with the new evidence.
12. Schema 0.4 and all Eggprobe public contracts remain untouched.

## 5. Durable isolated repro

Create an isolated, non-workspace reproduction package, for example:

```text
tools/repros/trippy-windows-1793/
├── README.md
├── src/main.rs
├── registry-0.13.0/Cargo.toml
└── upstream-candidate/Cargo.toml
```

Equivalent structure is acceptable if it keeps the evidence harness outside the
normal Eggprobe workspace.

The reproducer MUST:

- contain no Eggprobe runtime dependency;
- target `127.0.0.1`;
- use `trippy_core::Protocol::Udp`;
- use `PrivilegeMode::Privileged`;
- match the C005 builder configuration closely enough to exercise the same
  Windows receive path;
- perform a bounded single/few-round trace;
- print only minimal version/config/result markers;
- document that it must run in an elevated Windows process;
- make the dependency source obvious for every build.

Two required variants:

### Baseline

Exact crates.io:

```toml
trippy-core = "=0.13.0"
trippy-privilege = "=0.13.0"
```

Expected evidence: reproduce the C005/#1793 non-unwinding abort on the same
class of elevated Windows runner.

### Upstream candidate

First test exact fix commit:

```text
0b3c85bae2257915583d71392ea6d10c1cb4b75b
```

Then test the reviewed current master:

```text
c0c758eb1069151eafa6bd8227bb446c2b4d1506
```

Git dependencies are acceptable only inside this evidence harness and MUST NOT
be copied to Eggprobe production manifests.

If upstream master advances before implementation, record both the planned
reviewed master above and the actual tested master SHA.

## 6. Windows evidence execution

Use an isolated manually-triggered Windows evidence job or equivalent
short-lived qualification branch. Do not put the intentionally crashing 0.13.0
baseline into the ordinary always-on quality matrix.

The job/harness must:

1. record Windows runner image/OS;
2. record `rustc -Vv` and Cargo version;
3. prove the process is elevated before attempting privileged mode;
4. build/run the baseline as a child process so its abort does not prevent the
   evidence controller from recording its exit status;
5. capture bounded stderr/backtrace;
6. assert the baseline matches the expected crash family rather than accepting
   any failure;
7. build/run the exact #1793 fix commit under the same conditions;
8. run the candidate repeatedly (minimum 10 bounded loopback traces unless
   execution cost materially requires a documented alternative);
9. repeat against current upstream master;
10. preserve logs/run IDs in the C007 closure record.

Do not turn a baseline abort into a green "test" by discarding its signature.
The outer evidence job may succeed only after it explicitly verifies the
baseline failure and candidate non-abort conditions.

## 7. Candidate interpretation

### Case A — #1793 fix commit and current master both eliminate the abort

Close C007 as successful upstream-fix qualification.

Record:

- #1793 as the durable upstream issue;
- `0b3c85b` as the minimal known fix commit;
- tested master SHA;
- baseline/candidate run IDs and repeat counts;
- any normal recoverable trace result seen after the fix.

Update C006 so its publication gate requires an immutable release containing
`0b3c85b` or an explicitly verified equivalent.

Do not create a new upstream issue solely to ask for a release.

### Case B — exact fix commit passes but current master regresses

File or reopen upstream with a regression report referencing #1793, including:

- last known passing fix commit;
- tested failing master SHA;
- minimal reproducer;
- exact abort/typed failure;
- Windows/Rust environment.

C006 remains blocked on a published release from a reconciled good line.

### Case C — #1793 fix commit still reproduces the Eggprobe abort

The C005 failure is not fully explained by #1793.

File a new Trippy bug (or comment on/reopen #1793 if maintainers prefer) using
the repository's bug-report template.

Suggested title:

```text
Windows: privileged UDP trace still aborts with 0xC0000409 after #1793
```

The report must include:

- concise bug description;
- standalone reproduction command/code;
- expected recoverable behavior;
- actual abort code and bounded backtrace;
- Windows version/runner image;
- Rust version;
- Trippy baseline/fix/master SHAs;
- relationship to #1793 and why the new evidence is distinct.

Then update C006's blocker to the new upstream issue/fix.

## 8. Upstream communication rules

Trippy's current bug-report template asks for:

- description;
- reproduction steps/full command;
- expected behavior;
- environment information;
- additional context.

C007 must satisfy those fields if a new report is necessary.

Do not paste large raw Eggprobe CI logs into the issue body. Provide the minimal
standalone repro, concise crash excerpt, relevant run links, and full backtrace
as an attachment/link or collapsed follow-up where practical.

Do not claim #1793 is definitively the Eggprobe fix until C007's same-repro
candidate run proves it.

## 9. Planning/control-surface cleanup

C007 also owns the small planning hygiene noted after C005:

- remove the duplicate execution-order item number in `plans/registry.md`;
- rewrite the historical M005 execution-order sentence so it does not imply the
  pre-C005 closure alone is current Windows qualification;
- register C007 as the immediate ready native corrective;
- refine C006 from "generic unknown upstream fix" to
  "C007-qualified #1793 fix/equivalent + immutable published release";
- keep M006 blocked independently on the PMTU DF/PTB control gap.

This is documentation/control-surface reconciliation only, not a separate
implementation milestone.

## 10. In scope

- isolated standalone Trippy reproducer;
- evidence-only exact Git commit testing;
- elevated Windows baseline/candidate qualification;
- upstream #1793 correlation;
- new upstream issue only if the existing fix is insufficient/regressed;
- durable evidence/closure record;
- registry/roadmap/C006 blocker reconciliation.

## 11. Out of scope

- changing Eggprobe production Trippy pins;
- enabling Windows trace execution in Eggprobe;
- vendoring/forking Trippy;
- modifying Trippy upstream code;
- waiting for or publishing Trippy 0.14.0;
- PMTU work;
- ICMP/ping-async work;
- schema changes;
- release/publication work.

C006 owns production adoption after publication.

## 12. Verification

For Eggprobe planning/product tree after C007 evidence tooling lands:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo +stable audit
cargo run -p eggprobe-core --example generate-schemas --locked
```

Require no production `Cargo.lock` change from the evidence harness and no
schema diff.

Evidence harness verification must separately record its own dependency source
and lockfile/commit identity.

## 13. Acceptance criteria

C007 closes only when all are true:

1. a durable standalone elevated-Windows reproducer exists;
2. registry 0.13.0 reproduces the defining C005/#1793 crash signature;
3. the exact upstream #1793 fix commit is tested under the same conditions;
4. current upstream master is tested under the same conditions;
5. candidate runs are repeated/bounded and prove non-abort or expose a
   reproducible remaining failure;
6. the relationship between C005 and #1793 is classified as fixed, regressed,
   or distinct;
7. if distinct/regressed, an upstream issue/comment is filed with durable
   evidence;
8. if fixed, #1793 and `0b3c85b` become the durable C006 fix provenance;
9. production Eggprobe remains pinned to published 0.13.0 with Windows refusal;
10. ordinary Eggprobe hosted CI remains green;
11. schema regeneration has no diff;
12. registry/roadmap/C006 blocker wording is reconciled;
13. closure evidence records all relevant SHAs, run IDs, and upstream links.

## 14. Stop conditions

Stop and reassess if:

- the baseline cannot reproduce the C005/#1793 abort on an elevated Windows
  environment comparable to the original runner;
- the isolated repro accidentally depends on Eggprobe behavior;
- candidate testing requires changing production dependencies;
- upstream master changes trace semantics enough that the repro is no longer
  comparable;
- the observed failure is no longer memory-safety/abort-related and needs a
  different upstream issue;
- the fixed upstream line raises its MSRV or dependency/security posture such
  that C006 cannot plausibly consume it.

Failure to prove the fix does not justify enabling Windows tracing.

## 15. Closure evidence

Create:

- `plans/closure/native-path-host-diagnostics-corrective/007-status.md`

The closure record MUST include:

- planning baseline;
- reproducer paths/commit;
- exact Windows environment;
- elevation proof;
- Rust/Cargo versions;
- registry 0.13.0 baseline dependency identity and result;
- baseline abort exit code/signature/backtrace excerpt;
- exact #1793 fix commit dependency identity and result;
- tested master SHA and result;
- repeat counts;
- evidence workflow/run/job IDs;
- upstream issue/comment URL if any new communication was necessary;
- conclusion: #1793 fixed / regression / distinct defect;
- C006 blocker disposition;
- full Eggprobe verification results;
- schema no-diff proof;
- confirmation production pins/refusal were unchanged.

## 16. Handoff notes

This is the immediate actionable native-diagnostics work.

Do not file a duplicate upstream bug before running the already-landed #1793
fix against the Eggprobe reproducer.

Do not use the evidence-only Git dependency as a production shortcut. Even if
master is proven clean, C006 remains blocked until an immutable crates.io
release containing the qualified fix exists.

If #1793 proves to be the exact fix, the correct next state is not additional
Eggprobe implementation: close C007, keep C006 blocked on publication, and
leave the C005 Windows refusal in place.
