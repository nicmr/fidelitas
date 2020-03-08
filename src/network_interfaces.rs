use std::net::IpAddr;
use std::collections::{VecDeque};

#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub ip_addresses: Vec<IpAddr>,
}

#[cfg(unix)]
pub fn interfaces() -> Option<Vec<NetworkInterface>> {
    let unix_adapters = ifaces::Interface::get_all();

    match unix_adapters {
        Err(_)=> None,

        Ok(adapters) => {
            Some (
                adapters
                .iter()
                .filter_map(|adapter| {
                    match adapter.addr {
                        None => None,
                        Some(socket_addr) => {
                            let mut addresses = Vec::with_capacity(1);
                            addresses.push(socket_addr.ip());

                            Some(NetworkInterface {
                                name : adapter.name.clone(),
                                ip_addresses : addresses
                            })
                        }
                    }
                })
                .collect()
            )
        }
    }
}

#[cfg(windows)]
pub fn interfaces() -> Option<Vec<NetworkInterface>> {
    let win_adapters = ipconfig::get_adapters();

    match win_adapters {
        Err(_) => None,

        Ok(adapters) => {
            Some (
                adapters
                .iter()
                .map(|adapter|
                    NetworkInterface {
                        name : adapter.adapter_name().to_owned(),
                        ip_addresses : adapter.ip_addresses().to_vec(),
                    }
                )
                .collect()
            )
        }
    }
}

/// Selects one of the available network interfaces to determine the IP the application will use.
/// Currently only has a basic strategy for unix, will return None for windows and require a passed override_interface parameter.
pub fn select_network_interface (select_from: &Vec<NetworkInterface>, override_interface: Option<&str>) -> Option<NetworkInterface> {
    match override_interface {
        None => {
            select_from
            .iter()
            .filter(|a| a.name.starts_with("en") || a.name.starts_with("wl") )
            .next()
            .map(|a| a.clone())
        }
        Some(interface) => {
            select_from
            .iter()
            .filter(|a| a.name.starts_with(interface))
            .next()
            .map(|a| a.clone())
        }
    }
}

/// Sorts the available ip adresses so IpV4 adresses will be in the front and IpV6 in the back
pub fn v4_first (mut deque: VecDeque<IpAddr>, ip : &IpAddr) -> VecDeque<IpAddr> {
    match ip {
        IpAddr::V4(_) => deque.push_front(*ip),
        IpAddr::V6(_) => deque.push_back(*ip),
    }
    deque
}
