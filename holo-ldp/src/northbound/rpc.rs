//
// Copyright (c) The Holo Core Contributors
//
// See LICENSE for license details.
//

use std::net::{IpAddr, Ipv4Addr};
use std::sync::LazyLock as Lazy;

use holo_northbound::paths;
use holo_northbound::rpc::{Callbacks, CallbacksBuilder, Provider};
use holo_utils::yang::DataNodeRefExt;
use yang2::data::Data;

use crate::discovery;
use crate::instance::{Instance, InstanceUp};
use crate::neighbor::{self, Neighbor};
use crate::packet::messages::notification::StatusCode;

pub static CALLBACKS: Lazy<Callbacks<Instance>> = Lazy::new(load_callbacks);

// ===== callbacks =====

fn load_callbacks() -> Callbacks<Instance> {
    CallbacksBuilder::<Instance>::default()
        .path(paths::mpls_ldp_clear_peer::PATH)
        .rpc(|instance, args| {
            Box::pin(async move {
                let rpc = args.data.find_path(args.rpc_path).unwrap();

                // Parse input parameters.
                let (lsr_id, lspace_id) = (
                    rpc.get_ipv4_relative("./lsr-id"),
                    rpc.get_u16_relative("./label-space-id"),
                );

                // Clear peers.
                if let Instance::Up(instance) = instance {
                    clear_peers(instance, lsr_id, lspace_id);
                }

                Ok(())
            })
        })
        .path(paths::mpls_ldp_clear_hello_adjacency::PATH)
        .rpc(|instance, args| {
            Box::pin(async move {
                let rpc = args.data.find_path(args.rpc_path).unwrap();

                // Parse input parameters.
                let (nexthop_ifname, nexthop_addr, tnbr_addr) = (
                    rpc.get_string_relative(
                        "./hello-adjacency/link/next-hop-interface",
                    ),
                    rpc.get_ip_relative(
                        "./hello-adjacency/link/next-hop-address",
                    ),
                    rpc.get_ip_relative(
                        "./hello-adjacency/targeted/target-address",
                    ),
                );

                // Clear adjacencies.
                if let Instance::Up(instance) = instance {
                    clear_adjacencies(
                        instance,
                        nexthop_ifname,
                        nexthop_addr,
                        tnbr_addr,
                    );
                }

                Ok(())
            })
        })
        .path(paths::mpls_ldp_clear_peer_statistics::PATH)
        .rpc(|instance, args| {
            Box::pin(async move {
                let rpc = args.data.find_path(args.rpc_path).unwrap();

                // Parse input parameters.
                let (lsr_id, lspace_id) = (
                    rpc.get_ipv4_relative("./lsr-id"),
                    rpc.get_u16_relative("./label-space-id"),
                );

                // Clear peers.
                if let Instance::Up(instance) = instance {
                    clear_statistics(instance, lsr_id, lspace_id);
                }

                Ok(())
            })
        })
        .build()
}

// ===== impl Instance =====

impl Provider for Instance {
    fn callbacks() -> Option<&'static Callbacks<Instance>> {
        Some(&CALLBACKS)
    }
}

// ===== helper functions =====

fn clear_peers(
    instance: &mut InstanceUp,
    lsr_id: Option<Ipv4Addr>,
    lspace_id: Option<u16>,
) {
    for nbr_idx in instance.state.neighbors.indexes().collect::<Vec<_>>() {
        let nbr = &mut instance.state.neighbors[nbr_idx];

        // Skip uninitialized neighbors.
        if nbr.state == neighbor::fsm::State::NonExistent {
            continue;
        }

        // Filter by LSR-ID.
        if let Some(lsr_id) = lsr_id {
            if nbr.lsr_id != lsr_id {
                continue;
            }
        }
        if let Some(lspace_id) = lspace_id {
            if lspace_id != 0 {
                continue;
            }
        }

        // Send Shutdown notification.
        nbr.send_shutdown(&instance.state.msg_id, None);
        Neighbor::fsm(instance, nbr_idx, neighbor::fsm::Event::ErrorSent);
    }
}

fn clear_adjacencies(
    instance: &mut InstanceUp,
    nexthop_ifname: Option<String>,
    nexthop_addr: Option<IpAddr>,
    tnbr_addr: Option<IpAddr>,
) {
    for adj_idx in instance
        .state
        .ipv4
        .adjacencies
        .indexes()
        .collect::<Vec<_>>()
    {
        let adjacencies = &mut instance.state.ipv4.adjacencies;
        let adj = &adjacencies[adj_idx];

        // Filter by source.
        if let Some(iface_id) = adj.source.iface_id {
            if let Some(nexthop_ifname) = &nexthop_ifname {
                let (_, iface) =
                    instance.core.interfaces.get_by_id(iface_id).unwrap();
                if iface.name != *nexthop_ifname {
                    continue;
                }
            }
            if let Some(nexthop_addr) = &nexthop_addr {
                if adj.source.addr != *nexthop_addr {
                    continue;
                }
            }
        } else if let Some(tnbr_addr) = &tnbr_addr {
            if adj.source.addr != *tnbr_addr {
                continue;
            }
        }

        // Delete adjacency.
        discovery::adjacency_delete(instance, adj_idx, StatusCode::Shutdown);
    }
}

fn clear_statistics(
    instance: &mut InstanceUp,
    lsr_id: Option<Ipv4Addr>,
    lspace_id: Option<u16>,
) {
    for nbr in instance.state.neighbors.iter_mut() {
        // Filter by LSR-ID.
        if let Some(lsr_id) = lsr_id {
            if nbr.lsr_id != lsr_id {
                continue;
            }
        }
        if let Some(lspace_id) = lspace_id {
            if lspace_id != 0 {
                continue;
            }
        }

        // Clear neighbor statistics.
        nbr.statistics = Default::default();
    }
}
