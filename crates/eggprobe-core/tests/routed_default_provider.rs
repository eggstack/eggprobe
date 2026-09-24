//! C006 regression: a routed probe must succeed in a process that never
//! installed the rustls default CryptoProvider.
//!
//! This file is intentionally a separate integration target that never calls
//! `install_default`: each `tests/` target runs in its own process, while the
//! existing `routed_qualification` target installs the provider globally at
//! setup and would therefore mask the production gap this guards. Do not move
//! these cases into that target and do not add provider installation here.

use std::net::SocketAddr;

use eggprobe_core::{
    DurationMicros, EggressRoute, ExecutionPolicy, ProbeEngine, ProbePlan, ProbeSpec, ProbeStatus,
    RouteSpec, SchemaVersion, TargetSpec,
};
use tokio::net::TcpListener;

fn start_proxy() -> (eggress_embed::EggressHandle, SocketAddr) {
    let config = eggress_embed::EggressConfig::from_toml_str(
        "version = 1\n\n[[listeners]]\nname = \"proxy\"\nbind = \"127.0.0.1:0\"\nprotocols = [\"socks5\"]\n",
    )
    .unwrap();
    let handle = eggress_embed::EggressService::new(config)
        .start_blocking()
        .unwrap();
    let address = handle.bound_addresses().listeners[0].addr;
    (handle, address)
}

#[tokio::test]
async fn routed_tcp_succeeds_without_preinstalled_crypto_provider() {
    let (proxy, proxy_addr) = start_proxy();
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_port = target.local_addr().unwrap().port();
    let accepted = tokio::spawn(async move { target.accept().await.unwrap() });

    let plan = ProbePlan {
        schema_version: SchemaVersion::CURRENT,
        target: TargetSpec::new("127.0.0.1", Some(target_port)).unwrap(),
        route: RouteSpec::Eggress(EggressRoute {
            expression: format!("socks5://{proxy_addr}"),
        }),
        probes: vec![ProbeSpec::Tcp { port: target_port }],
        execution: ExecutionPolicy {
            deadline: DurationMicros::from_micros(8_000_000),
            ..ExecutionPolicy::default()
        },
        assertions: vec![],
    };
    let report = ProbeEngine::default().execute(plan).await;

    assert_eq!(report.probes[0].status, ProbeStatus::Ok, "{report:?}");
    let json: serde_json::Value = serde_json::from_str(&report.to_json().unwrap()).unwrap();
    assert_eq!(
        json["route"],
        serde_json::json!({"kind": "eggress"}),
        "{json}"
    );
    drop(accepted.await.unwrap());
    proxy.shutdown_blocking().unwrap();
}
