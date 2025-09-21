use std::{
    collections::HashMap,
    fmt::Display,
    net::IpAddr,
    sync::{Arc, Mutex},
};

use eyre::Result;
use ipnet::IpNet;
use itertools::Itertools;
use tokio_util::task::TaskTracker;
use tracing::instrument;

use crate::types::{ByteCount, DisplayFields, display_list, display_map};

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
    data: ByteCount,
    packets: u64,
    errors: u64,
}

impl Display for Counters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("data", &self.data)
            .display("packets", &self.packets)
            .display("errors", &self.errors)
            .finish(f)
    }
}

#[derive(Debug)]
pub struct Traffic {
    total: Counters,
    delta: Counters,
}

impl Display for Traffic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display("total", &self.total)
            .display("delta", &self.delta)
            .finish(f)
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
        DisplayFields::default()
            .display("inbound", &self.inbound)
            .display("outbound", &self.outbound)
            .display("mtu", &self.mtu)
            .display_if_some("mac_address", &self.mac_address)
            .display("subnets", display_list(&self.subnets))
            .finish(f)
    }
}

impl From<&sysinfo::NetworkData> for NetworkInterface {
    fn from(value: &sysinfo::NetworkData) -> Self {
        Self {
            inbound: Traffic {
                total: Counters {
                    data: value.total_received().into(),
                    packets: value.total_packets_received(),
                    errors: value.total_errors_on_received(),
                },
                delta: Counters {
                    data: value.received().into(),
                    packets: value.packets_received(),
                    errors: value.errors_on_received(),
                },
            },
            outbound: Traffic {
                total: Counters {
                    data: value.total_transmitted().into(),
                    packets: value.total_packets_transmitted(),
                    errors: value.total_errors_on_transmitted(),
                },
                delta: Counters {
                    data: value.transmitted().into(),
                    packets: value.packets_transmitted(),
                    errors: value.errors_on_transmitted(),
                },
            },
            mtu: value.mtu(),
            mac_address: (!value.mac_address().is_unspecified())
                .then(|| value.mac_address().to_string()),
            subnets: value.ip_networks().iter().map(Subnet::from).collect_vec(),
        }
    }
}

#[derive_where::derive_where(Debug)]
pub struct Network {
    #[derive_where(skip)]
    networks: Arc<Mutex<sysinfo::Networks>>,

    #[derive_where(skip)]
    task_tracker: TaskTracker,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        DisplayFields::default()
            .display_if_some("hostname", &self.hostname())
            .display("interfaces", display_map(&self.interfaces()))
            .finish(f)
    }
}

impl Network {
    #[instrument(name = "network", skip_all)]
    pub async fn new(task_tracker: TaskTracker) -> Result<Self> {
        let networks = task_tracker
            .spawn_blocking(|| sysinfo::Networks::new_with_refreshed_list())
            .await?;

        Ok(Self {
            networks: Arc::new(Mutex::new(networks)),
            task_tracker,
        })
    }

    pub fn hostname(&self) -> Option<String> {
        sysinfo::System::host_name()
    }

    pub async fn refresh_interfaces(
        &self,
        rescan: bool,
    ) -> Result<HashMap<String, NetworkInterface>> {
        let networks = self.networks.clone();
        let result = self
            .task_tracker
            .spawn_blocking(move || {
                let mut networks = networks.lock().unwrap();
                networks.refresh(rescan);
                networks
                    .list()
                    .iter()
                    .map(|(name, data)| (name.clone(), data.into()))
                    .collect::<HashMap<_, _>>()
            })
            .await?;
        Ok(result)
    }

    pub fn interfaces(&self) -> HashMap<String, NetworkInterface> {
        let networks = self.networks.lock().unwrap();
        networks
            .list()
            .iter()
            .map(|(name, data)| (name.clone(), data.into()))
            .collect::<HashMap<_, _>>()
    }
}
