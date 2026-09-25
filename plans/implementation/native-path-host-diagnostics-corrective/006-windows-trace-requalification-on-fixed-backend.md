# Native Path and Host Diagnostics Corrective C006 — Windows Trace Re-qualification on Fixed Backend

Status: blocked

Repository baseline: `53226a3`

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

A published, immutable crates.io release of `trippy-core` (plus its
`trippy-privilege` line) containing a fix for the Windows privileged-run
memory corruption, consumable as an exact pin under Rust 1.89 with no new
advisories. A Git SHA, fork, or vendored patch is not an accepted production
bridge (same rule as C003).

## 3. Entry criteria

1. Upstream issue filed against `fujiapple852/trippy` with the C005
   backtrace, the loopback repro (`eggprobe trace 127.0.0.1` on an elevated
   Windows host via `trippy-core 0.13.0` privileged UDP), and confirmation
   from the maintainer that the corruption is addressed.
2. Published release containing the fix, reviewed against Rust 1.89,
   dependency/security policy, and the M005 silent-attempt/no-reverse-DNS
   qualification (as done for 0.13.0 in the historical M005 closure).
3. No schema change required by the adoption.

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
