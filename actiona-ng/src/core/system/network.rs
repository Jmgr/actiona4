use std::{
    collections::HashMap,
    fmt::Display,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use humansize::BINARY;
use ipnet::IpNet;
use itertools::Itertools;

#[derive(Debug)]
pub struct Subnet {
    address: IpAddr,
    prefix: u8,
}

impl Display for Subnet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Ok(ipnet) = IpNet::new(self.address, self.prefix) {
            write!(f, "{}", ipnet)
        } else {
            write!(f, "<INVALID>")
        }
    }
}

impl From<&sysinfo::IpNetwork> for Subnet {
    fn from(value: &sysinfo::IpNetwork) -> Self {
        Self {
            address: value.addr.clone(),
            prefix: value.prefix,
        }
    }
}

#[derive(Debug)]
pub struct Counters {
    data: u64,
    packets: u64,
    errors: u64,
}

impl Display for Counters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(data: {}, packets: {}, errors: {})",
            humansize::format_size(self.data, BINARY),
            self.packets,
            self.errors
        )
    }
}

#[derive(Debug)]
pub struct Traffic {
    total: Counters,
    delta: Counters,
}

impl Display for Traffic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(total: {}, delta: {})", self.total, self.delta)
    }
}

#[derive(Debug)]
pub struct NetworkInterface {
    inbound: Traffic,
    outbound: Traffic,
    mtu: u64,
    mac_address: Option<String>,
    subnets: Vec<Subnet>,
}

impl Display for NetworkInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(inbound: {}, outbound: {}, mtu: {}, mac address: {}, subnets: [{}])",
            self.inbound,
            self.outbound,
            self.mtu,
            self.mac_address.as_deref().unwrap_or(""),
            self.subnets.iter().join(", ")
        )
    }
}

impl From<&sysinfo::NetworkData> for NetworkInterface {
    fn from(value: &sysinfo::NetworkData) -> Self {
        Self {
            inbound: Traffic {
                total: Counters {
                    data: value.total_received(),
                    packets: value.total_packets_received(),
                    errors: value.total_errors_on_received(),
                },
                delta: Counters {
                    data: value.received(),
                    packets: value.packets_received(),
                    errors: value.errors_on_received(),
                },
            },
            outbound: Traffic {
                total: Counters {
                    data: value.total_transmitted(),
                    packets: value.total_packets_transmitted(),
                    errors: value.total_errors_on_transmitted(),
                },
                delta: Counters {
                    data: value.transmitted(),
                    packets: value.packets_transmitted(),
                    errors: value.errors_on_transmitted(),
                },
            },
            mtu: value.mtu(),
            mac_address: (!value.mac_address().is_unspecified())
                .then(|| value.mac_address().to_string()),
            subnets: value
                .ip_networks()
                .iter()
                .map(|ip_network| ip_network.into())
                .collect_vec(),
        }
    }
}

#[derive_where::derive_where(Debug)]
pub struct Network {
    #[derive_where(skip)]
    networks: Arc<Mutex<sysinfo::Networks>>,
}

impl Default for Network {
    fn default() -> Self {
        Self {
            networks: Arc::new(Mutex::new(sysinfo::Networks::new())),
        }
    }
}

impl Network {
    pub fn hostname(&self) -> Option<String> {
        sysinfo::System::host_name()
    }

    pub fn interfaces(&self) -> HashMap<String, NetworkInterface> {
        let mut networks = self.networks.lock().unwrap();
        networks.refresh(true);
        let interfaces = networks
            .list()
            .iter()
            .map(|(name, data)| (name.clone(), data.into()))
            .collect::<HashMap<_, _>>();

        interfaces
    }
}
