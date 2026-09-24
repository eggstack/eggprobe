//! Backend-neutral request and observation types for native diagnostics.
//!
//! Platform adapters are deliberately kept behind this crate. This initial
//! substrate does not perform native operations; capabilities are explicit.

#![deny(unsafe_code)]
#![warn(missing_docs)]

use std::{future::Future, net::IpAddr, pin::Pin};

/// Native operation requested by the core engine.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeRequest {
    /// Inspect local path selection and route metadata for one address.
    Route {
        /// Address inspected.
        target: IpAddr,
    },
    /// Send one bounded ICMP echo request.
    IcmpEcho {
        /// Destination address.
        target: IpAddr,
        /// Echo sequence.
        sequence: u16,
        /// Payload length.
        payload_bytes: u16,
    },
    /// Send one direct UDP datagram.
    Udp {
        /// Destination socket.
        target: std::net::SocketAddr,
        /// Bounded request payload.
        payload: Vec<u8>,
    },
    /// Trace one bounded path.
    Trace {
        /// Destination address.
        target: IpAddr,
        /// Maximum hop count.
        max_hops: u8,
        /// Maximum attempts at each hop.
        attempts_per_hop: u8,
    },
    /// Probe one packet size for active path-MTU discovery.
    PathMtu {
        /// Destination address.
        target: IpAddr,
        /// Packet size to probe.
        packet_bytes: u16,
    },
}

/// Stable result category independent of an operating-system error enum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeErrorKind {
    /// The local process lacks the required permission.
    PermissionDenied,
    /// The operation is not available on this platform/backend.
    Unsupported,
    /// The finite operation deadline elapsed.
    Timeout,
    /// The remote network or host reported an unreachable condition.
    Unreachable,
    /// A local operation failed.
    Io,
}

/// A bounded, dependency-neutral backend failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeError {
    /// Stable category.
    pub kind: NativeErrorKind,
}

/// Backend-neutral observation. Detailed family results are added with their
/// owning capability milestones.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeObservation {
    /// A capability is absent on this platform.
    Unsupported,
    /// Request accepted by the backend; capability milestones refine this.
    Completed,
}

/// Boxed future returned by an injectable native backend.
pub type NativeFuture<'a> =
    Pin<Box<dyn Future<Output = Result<NativeObservation, NativeError>> + Send + 'a>>;

/// Injectable native operations used by core normalization and tests.
pub trait NativeBackend: Send + Sync {
    /// Execute one native request, or report an explicit capability/error state.
    fn execute(&self, request: NativeRequest) -> NativeFuture<'_>;
}

/// Platform-neutral interface facts needed by the route probe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InterfaceObservation {
    /// OS-assigned interface index.
    pub index: u32,
    /// System interface name.
    pub name: String,
    /// Addresses assigned to the interface.
    pub addresses: Vec<IpAddr>,
    /// Whether the interface is administratively up.
    pub up: bool,
    /// Interface MTU in bytes, when available.
    pub mtu: Option<u32>,
}

/// Read-only route candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteObservation {
    /// Destination prefix, for example `192.0.2.0/24`.
    pub destination: String,
    /// Egress interface index when exposed.
    pub interface_index: Option<u32>,
    /// Gateway when exposed.
    pub gateway: Option<IpAddr>,
    /// Route metric when exposed.
    pub metric: Option<u32>,
    /// Routing table identifier when exposed.
    pub table: Option<u32>,
    /// Route protocol when exposed.
    pub protocol: Option<String>,
    /// Route scope when exposed.
    pub scope: Option<String>,
}

/// Result of source-address observation and route-table correlation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouteInspection {
    /// Kernel-selected source address from a UDP connect with no payload.
    pub source: Option<IpAddr>,
    /// Interface matched by the observed source address.
    pub interface: Option<InterfaceObservation>,
    /// Best-prefix route candidates; these are not authoritative selection.
    pub candidates: Vec<RouteObservation>,
    /// Whether more equally specific candidates were present than the cap.
    pub candidates_truncated: bool,
    /// True when multiple equally specific candidates remain.
    pub ambiguous: bool,
}

/// Inspect local interface and route metadata for one resolved destination.
///
/// # Errors
///
/// Returns a bounded native error when the OS route snapshot cannot be read.
pub async fn inspect_route(target: IpAddr) -> Result<RouteInspection, NativeError> {
    let bind: std::net::SocketAddr = match target {
        IpAddr::V4(_) => std::net::SocketAddr::new(IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), 0),
        IpAddr::V6(_) => std::net::SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED), 0),
    };
    let socket = tokio::net::UdpSocket::bind(bind)
        .await
        .map_err(|error| normalize_io(&error))?;
    socket
        .connect(std::net::SocketAddr::new(target, 33434))
        .await
        .map_err(|error| normalize_io(&error))?;
    let source = socket.local_addr().ok().map(|address| address.ip());
    drop(socket);

    let interfaces = netdev::get_interfaces();
    let interface = source.and_then(|source| {
        interfaces
            .iter()
            .find(|item| {
                item.ipv4
                    .iter()
                    .any(|network| IpAddr::V4(network.addr()) == source)
                    || item
                        .ipv6
                        .iter()
                        .any(|network| IpAddr::V6(network.addr()) == source)
            })
            .map(|item| InterfaceObservation {
                index: item.index,
                name: item.name.clone(),
                addresses: item.ip_addrs().into_iter().take(64).collect(),
                up: item.is_up(),
                mtu: item.mtu,
            })
    });

    let mut routes = netroute::list_routes().map_err(|error| normalize_io(&error))?;
    routes.retain(|route| {
        prefix_contains(route.destination.addr, route.destination.prefix_len, target)
    });
    routes.sort_by(|left, right| {
        right
            .destination
            .prefix_len
            .cmp(&left.destination.prefix_len)
            .then_with(|| left.destination.addr.cmp(&right.destination.addr))
            .then_with(|| left.ifindex.cmp(&right.ifindex))
            .then_with(|| left.metric.cmp(&right.metric))
    });
    let best_prefix = routes.first().map(|route| route.destination.prefix_len);
    routes.retain(|route| Some(route.destination.prefix_len) == best_prefix);
    let candidates_truncated = routes.len() > 32;
    let candidates = routes
        .into_iter()
        .take(32)
        .map(|route| RouteObservation {
            destination: route.destination.to_string(),
            interface_index: route.ifindex,
            gateway: route.gateway,
            metric: route.metric,
            table: route.table,
            protocol: route.protocol.map(|value| format!("{value:?}")),
            scope: route.scope.map(|value| format!("{value:?}")),
        })
        .collect::<Vec<_>>();
    let ambiguous = candidates.len() > 1;
    Ok(RouteInspection {
        source,
        interface,
        candidates,
        candidates_truncated,
        ambiguous,
    })
}

fn normalize_io(error: &std::io::Error) -> NativeError {
    let kind = match error.kind() {
        std::io::ErrorKind::PermissionDenied => NativeErrorKind::PermissionDenied,
        std::io::ErrorKind::TimedOut => NativeErrorKind::Timeout,
        std::io::ErrorKind::NetworkUnreachable | std::io::ErrorKind::HostUnreachable => {
            NativeErrorKind::Unreachable
        }
        _ => NativeErrorKind::Io,
    };
    NativeError { kind }
}

fn prefix_contains(prefix: IpAddr, bits: u8, target: IpAddr) -> bool {
    match (prefix, target) {
        (IpAddr::V4(prefix), IpAddr::V4(target)) if bits <= 32 => {
            let mask = if bits == 0 {
                0
            } else {
                u32::MAX << (32 - bits)
            };
            u32::from(prefix) & mask == u32::from(target) & mask
        }
        (IpAddr::V6(prefix), IpAddr::V6(target)) if bits <= 128 => {
            let mask = if bits == 0 {
                0
            } else {
                u128::MAX << (128 - bits)
            };
            u128::from(prefix) & mask == u128::from(target) & mask
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_backend_preserves_completed_and_permission_outcomes() {
        let request = NativeRequest::Route {
            target: IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        };
        let success = FakeBackend {
            result: Ok(NativeObservation::Completed),
        }
        .execute(request.clone())
        .await;
        assert_eq!(success, Ok(NativeObservation::Completed));

        let denied = FakeBackend {
            result: Err(NativeError {
                kind: NativeErrorKind::PermissionDenied,
            }),
        }
        .execute(request.clone())
        .await;
        assert_eq!(
            denied,
            Err(NativeError {
                kind: NativeErrorKind::PermissionDenied,
            })
        );

        let unsupported = FakeBackend {
            result: Ok(NativeObservation::Unsupported),
        }
        .execute(request)
        .await;
        assert_eq!(unsupported, Ok(NativeObservation::Unsupported));
    }

    #[test]
    fn prefix_match_is_family_safe_and_handles_default_route() {
        assert!(prefix_contains(
            "0.0.0.0".parse().unwrap(),
            0,
            "203.0.113.8".parse().unwrap()
        ));
        assert!(!prefix_contains(
            "192.0.2.0".parse().unwrap(),
            24,
            "2001:db8::1".parse().unwrap()
        ));
    }
}

/// Deterministic backend for contract and engine tests.
#[derive(Clone, Debug)]
pub struct FakeBackend {
    /// Result returned for each request.
    pub result: Result<NativeObservation, NativeError>,
}

impl NativeBackend for FakeBackend {
    fn execute(&self, _request: NativeRequest) -> NativeFuture<'_> {
        let result = self.result.clone();
        Box::pin(async move { result })
    }
}
