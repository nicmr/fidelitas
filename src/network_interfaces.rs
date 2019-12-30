use ifaces;
use std::fmt;
use std::fmt::Write;

#[derive(Clone, Debug)]
struct PrettyInterface (pub ifaces::Interface);

impl fmt::Display for PrettyInterface{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let addr = {
            match self.0.addr {
                Some(address) => {
                    let mut s = String::new();
                    write!(s, "{}", address)?;
                    s
                }
                None => String::from("Unknown")
            }
        };
        write! (
            f,
            "Interface {{ name: {} Kind: {:?} Address: {} }}",
            self.0.name,
            self.0.kind,
            addr
        )
    }
}

fn pretty_acc(mut s: String, iface: PrettyInterface) -> String{
    // unwrap should never fail as we're writing to a string
    write!(s, "\n{}", iface).unwrap();
    s
}

pub fn pretty_print(interfaces: Vec<ifaces::Interface>) -> String {
    interfaces
        .into_iter()
        .map(|a| PrettyInterface(a))
        .fold(String::from(""), pretty_acc)
}

pub fn select_network_interface (from: Vec<ifaces::Interface>, override_interface: Option<&str>) -> Option<ifaces::Interface> {
    match override_interface {
        None => {
            from
            .into_iter()
            .filter(|a| a.name.starts_with("en") || a.name.starts_with("wl") )
            .next()
        }
        Some(interface) => {
            from
            .into_iter()
            .filter(|a| a.name.starts_with(interface))
            .next()
        }
    }
}

pub fn ipv4 () -> Result<Vec<ifaces::Interface>, std::io::Error> {
    let interfaces = ifaces::ifaces()?
        .into_iter()
        .filter(|a| a.kind == ifaces::Kind::Ipv4)
        .collect();
    Ok(interfaces)
}