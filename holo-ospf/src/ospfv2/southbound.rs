//
// Copyright (c) The Holo Core Contributors
//
// See LICENSE for license details.
//

use holo_utils::ip::{IpAddrKind, IpNetworkKind};
use ipnetwork::Ipv4Network;

use crate::area::Area;
use crate::debug::Debug;
use crate::instance::InstanceUpView;
use crate::interface::Interface;
use crate::southbound::rx::SouthboundRxVersion;
use crate::version::Ospfv2;

// ===== impl Ospfv2 =====

impl SouthboundRxVersion<Self> for Ospfv2 {
    fn process_ifindex_update(
        _iface: &mut Interface<Self>,
        _area: &Area<Self>,
        _instance: &InstanceUpView<'_, Self>,
    ) {
        // Nothing to do.
    }

    fn process_addr_add(
        iface: &mut Interface<Self>,
        addr: Ipv4Network,
        unnumbered: bool,
    ) {
        if iface.system.primary_addr.is_none() {
            Debug::<Self>::InterfacePrimaryAddrSelect(&iface.name, &addr).log();

            // Mark address as the primary one.
            iface.system.primary_addr = Some(addr);
            iface.system.unnumbered = unnumbered;
        }
    }

    fn process_addr_del(iface: &mut Interface<Self>, addr: Ipv4Network) {
        if iface.system.primary_addr == Some(addr) {
            Debug::<Self>::InterfacePrimaryAddrDelete(&iface.name).log();

            // Remove primary address.
            iface.system.primary_addr = None;

            // Try to find other address to select as primary.
            if let Some(addr) = iface
                .system
                .addr_list
                .iter()
                .find(|addr| addr.ip().is_usable())
            {
                Debug::<Self>::InterfacePrimaryAddrSelect(&iface.name, addr)
                    .log();

                iface.system.primary_addr = Some(*addr);
            }
        }
    }
}
