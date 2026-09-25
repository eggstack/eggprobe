//! Isolated Trippy #1793 Windows reproducer (C007 evidence harness).
//!
//! CANDIDATE-FIX variant: exact upstream #1793 fix commit `0b3c85b`.
//!
//! Mirrors the C005 production builder
//! (`eggprobe-native::trace_path`): privileged UDP loopback trace with
//! `drop_privileges(false)`, classic multipath, fixed source port. Prints
//! only minimal `REPRO ...` markers. Must run elevated on Windows; exits `2`
//! without tracing when privilege is not effective. Contains no Eggprobe
//! runtime dependency. See `../README.md`.

use std::cell::Cell;
use std::net::IpAddr;
use std::time::Duration;

/// Dependency identity marker for this variant (human label only; the
/// authoritative identity is the variant `Cargo.lock`).
const VARIANT: &str = "candidate-fix=upstream-0b3c85b";

/// Canonical loopback target: no public Internet target is required.
const TARGET: &str = "127.0.0.1";

/// Bounded few-round, few-TTL trace: exactly the C005 qualifying smoke shape
/// (`trace 127.0.0.1 --max-hops 3` with default 3 attempts and the engine's
/// ~10 s per-round budget). Every other builder field keeps the trippy
/// default, identical to the C005 production adapter.
const MAX_TTL: u8 = 3;
const MAX_ROUNDS: usize = 3;

/// Per-trace source-port identity (same base as the C005 adapter).
const SRC_PORT: u16 = 43534;

fn main() {
    println!("REPRO variant={VARIANT}");
    println!("REPRO target={TARGET} protocol=Udp privilege=Privileged max_ttl={MAX_TTL} max_rounds={MAX_ROUNDS}");
    let privilege =
        trippy_privilege::Privilege::discover().expect("privilege discovery must succeed");
    println!("REPRO has_privileges={}", privilege.has_privileges());
    if !privilege.has_privileges() {
        eprintln!("REPRO NOT_ELEVATED refusing privileged trace");
        std::process::exit(2);
    }
    let target: IpAddr = TARGET.parse().expect("loopback target must parse");
    let tracer = trippy_core::Builder::new(target)
        .privilege_mode(trippy_core::PrivilegeMode::Privileged)
        .drop_privileges(false)
        .protocol(trippy_core::Protocol::Udp)
        .multipath_strategy(trippy_core::MultipathStrategy::Classic)
        .port_direction(trippy_core::PortDirection::FixedSrc(trippy_core::Port(
            SRC_PORT,
        )))
        .first_ttl(1)
        .max_ttl(MAX_TTL)
        .max_rounds(Some(MAX_ROUNDS))
        .read_timeout(Duration::from_millis(100))
        .max_round_duration(Duration::from_secs(10))
        .build()
        .expect("tracer build must succeed");
    println!("REPRO tracer_built");
    let rounds = Cell::new(0_u32);
    match tracer.run_with(|_round| {
        rounds.set(rounds.get() + 1);
    }) {
        Ok(()) => println!("REPRO RESULT completed rounds={}", rounds.get()),
        Err(error) => println!(
            "REPRO RESULT typed_error rounds={} error={error:?}",
            rounds.get()
        ),
    }
}
