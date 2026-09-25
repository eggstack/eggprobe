# Native Path and Host Diagnostics Corrective C006 — Windows Trace Re-qualification on Fixed Backend

Status: blocked

Repository baseline: `53226a3`

Current blocker refinement baseline: `f2167ee9332d94bf71c06babc183d87b059cc310`

C007 Case-C refinement baseline: this plan is updated by the C007 closure
(`plans/closure/native-path-host-diagnostics-corrective/007-status.md`),
which proved the #1793 fix insufficient and filed upstream issue #1881.

Source roadmap:

- `plans/subsystems/native-path-host-diagnostics-roadmap.md`

Corrective authority:

- `plans/closure/native-path-host-diagnostics-corrective/005-status.md`
  (Windows abort forensics and fail-safe refusal)

Applicable ADRs:

- `plans/adrs/ADR-0001-json-first-core-and-transport-ownership.md`
- `plans/adrs/ADR-0003-native-diagnostics-platform-and-subject-boundary.md`

Primary work class: upstream-blocked capability re-qualification

## 1. Objective

Re-qualify live Windows traceroute execution once a published Trippy backend
release fixes the privileged-run heap corruption documented in the C005
closure (fail-fast abort while dropping a parsed `UnknownExtension` on an
elevated host). Until then, Eggprobe's version-pinned `Unsupported` refusal
in `current_privilege_mode()` stands and MUST NOT be weakened into attempting
the aborting path.

## 2. Blocker / readiness gate

C007 is closed:

- `plans/closure/native-path-host-diagnostics-corrective/007-status.md`
  (Case C: the exact #1793 fix commit `0b3c85b` aborts 10/10 and current
  master `c0c758e` aborts 10/10 with the identical defining signature, so
  #1793 does not explain the C005 failure).

The durable upstream reference is now the C007-filed follow-up:

- `https://github.com/fujiapple852/trippy/issues/1881`
  (`Windows: privileged UDP trace still aborts with 0xC0000409 after #1793`).

C006 requires a published, immutable crates.io release of `trippy-core`
plus the matching `trippy-privilege` line containing the fix for **#1881**
(or an explicitly reconciled equivalent), consumable as an exact pin under
Rust 1.89 with no new advisories. A release containing only the #1793 fix
(`0b3c85b`) is proven insufficient and does not satisfy this gate.

A Git SHA, fork, or vendored patch is permitted only inside C007's isolated
evidence harness. It is not an accepted Eggprobe production bridge (same rule
as C003).

## 3. Entry criteria

1. C007 closure records that the C005 reproducer fails on registry
   `trippy-core 0.13.0` and still aborts on upstream fix commit
   `0b3c85b` and reconciled current master `c0c758e` (Case C), with the
   distinct defect tracked by upstream issue #1881.
2. Upstream issue #1881 is the durable upstream reference (#1793 is retained
   as related context, not as the fix).
3. A published release containing the #1881 fix exists and is
   reviewed against Rust 1.89, dependency/security policy, and the M005
   silent-attempt/no-reverse-DNS qualification.
4. No schema change is required by the adoption.

## 4. Expected work (after unblock)

1. Bump the exact `trippy-core`/`trippy-privilege` pins together; re-run
   duplicate/dependency/security review (single privilege implementation).
2. Flip the Windows `privileged_supported` gate in
   `current_privilege_mode()` back on for the qualified release line only;
   keep the refusal for any line without the fix.
3. Extend host-aware smokes: elevated Windows must demonstrate successful
   structured loopback evidence (or a typed backend error — never an abort);
   non-elevated Windows keeps the `PermissionDenied` arm.
4. Re-run the full verification family plus a same-head hosted matrix with
   per-OS dispositions (macOS unprivileged success, Linux
   privileged-or-denied per runner, Windows privileged success).
5. Close with a closure record; update M005 to fully qualified and retire
   this plan.

## 5. Out of scope

- Changing the trace protocol, evidence model, or schema to dodge the
  backend defect.
- Adopting the fix via fork/Git pin in production code.
- PMTU work (M006) and ICMP work (M003/C003), which keep their own gates.

## 6. Stop conditions

Stop and reassess if no upstream fix materializes, if the fixed release
cannot build under Rust 1.89, or if the fixed backend changes trace
semantics (silent attempts, termination, redaction) incompatibly.
