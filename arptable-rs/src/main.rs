use std::collections::BTreeMap;
use std::ffi::c_void;
use std::ptr::null;
use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;
use windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER;
use windows::Win32::NetworkManagement::IpHelper::IP_ADAPTER_ADDRESSES_LH;
use windows::Win32::NetworkManagement::IpHelper::MIB_IPNET_TYPE;
use windows::Win32::NetworkManagement::IpHelper::{GetIpNetTable, MIB_IPNETTABLE};
use windows::Win32::Networking::WinSock::{SOCKADDR_IN, SOCKADDR_IN6};

pub const AF_UNSPEC: u32 = 0;

#[repr(transparent)]
pub struct GetAdaptersAdressesFlags(pub u32);

#[link(name = "iphlpapi")]
unsafe extern "system" {
    pub fn GetAdaptersAddresses(
        family: u32,
        flags: GetAdaptersAdressesFlags,
        reserved: *const c_void,
        adapteraddresses: *mut IP_ADAPTER_ADDRESSES_LH,
        sizepointer: *mut u32,
    ) -> u32;
}

#[repr(C)]
pub struct ArpTableEntry {
    pub interface_index: u32,
    pub mac_address_len: u32,
    pub mac_address: [u8; 8],
    pub dw_addr: u32,
    pub state: MIB_IPNET_TYPE,
}

pub struct NetworkInterfaceInfo {
    pub name: String,
    pub description: String,
    pub if_index: u32,
    pub ips: Vec<String>,
    pub dns_servers: Vec<String>,
    pub arp_entries: Vec<ArpTableEntry>,
}

fn arp_type_to_str(t: MIB_IPNET_TYPE) -> &'static str {
    match t.0 {
        1 => "Other",
        2 => "Invalid",
        3 => "Dynamic",
        4 => "Static",
        _ => "Unknown",
    }
}

pub fn get_network_interfaces() -> Vec<(NetworkInterfaceInfo)> {
    unsafe {
        let mut buffer_length: u32 = 0;
        let ret = GetAdaptersAddresses(
            AF_UNSPEC,
            GetAdaptersAdressesFlags(0),
            null(),
            std::ptr::null_mut(),
            &mut buffer_length,
        );

        if ret != ERROR_BUFFER_OVERFLOW.0 {
            panic!("Failed to get buffer size: {}", ret);
        }

        let mut buffer = vec![0u8; buffer_length as usize];
        let adapters = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        let ret = GetAdaptersAddresses(
            AF_UNSPEC,
            GetAdaptersAdressesFlags(0),
            null(),
            adapters,
            &mut buffer_length,
        );

        if ret != 0 {
            panic!("GetAdaptersAddresses failed: {}", ret);
        }

        let mut result = Vec::new();
        let mut adapter = adapters;
        while !adapter.is_null() {
            let name = if !(*adapter).FriendlyName.is_null() {
                let mut len = 0;
                let p = (*adapter).FriendlyName.0;
                while *p.add(len) != 0 {
                    len += 1;
                }
                String::from_utf16_lossy(std::slice::from_raw_parts(p, len))
            } else {
                String::new()
            };

            let description = if !(*adapter).Description.is_null() {
                let mut len = 0;
                while *(*adapter).Description.0.add(len) != 0 {
                    len += 1;
                }
                let slice = std::slice::from_raw_parts((*adapter).Description.0, len);
                String::from_utf16_lossy(slice)
            } else {
                String::new()
            };

            let if_index = (*adapter).Anonymous1.Anonymous.IfIndex;

            let mut ips = Vec::new();
            let mut ua = (*adapter).FirstUnicastAddress;
            while !ua.is_null() {
                let sa = (*ua).Address.lpSockaddr;
                if !sa.is_null() {
                    match (*sa).sa_family {
                        AF_INET => {
                            let sin = *(sa as *const SOCKADDR_IN);
                            ips.push(
                                std::net::Ipv4Addr::from(u32::from_be(sin.sin_addr.S_un.S_addr))
                                    .to_string(),
                            );
                        }
                        AF_INET6 => {
                            let sin6 = *(sa as *const SOCKADDR_IN6);
                            ips.push(std::net::Ipv6Addr::from(sin6.sin6_addr.u.Byte).to_string());
                        }
                        _ => {}
                    }
                }
                ua = (*ua).Next;
            }

            let mut dns_servers = Vec::new();
            let mut dns = (*adapter).FirstDnsServerAddress;
            while !dns.is_null() {
                let sa = (*dns).Address.lpSockaddr;
                if !sa.is_null() {
                    match (*sa).sa_family {
                        AF_INET => {
                            let sin = *(sa as *const SOCKADDR_IN);
                            dns_servers.push(
                                std::net::Ipv4Addr::from(u32::from_be(sin.sin_addr.S_un.S_addr))
                                    .to_string(),
                            );
                        }
                        AF_INET6 => {
                            let sin6 = *(sa as *const SOCKADDR_IN6);
                            dns_servers
                                .push(std::net::Ipv6Addr::from(sin6.sin6_addr.u.Byte).to_string());
                        }
                        _ => {}
                    }
                }
                dns = (*dns).Next;
            }

            result.push(NetworkInterfaceInfo {
                name,
                description,
                if_index,
                ips,
                dns_servers,
                arp_entries: Vec::new(),
            });
            adapter = (*adapter).Next;
        }
        result
    }
}

pub fn get_arp_table() -> BTreeMap<u32, Vec<ArpTableEntry>> {
    unsafe {
        let mut bytes_needed: u32 = 0;

        let ret = GetIpNetTable(None, &mut bytes_needed, false);
        if ret != ERROR_INSUFFICIENT_BUFFER.0 {
            panic!("Expected ERROR_INSUFFICIENT_BUFFER but got {}", ret);
        }

        let mut buffer = vec![0u8; bytes_needed as usize];
        let table_ptr = buffer.as_mut_ptr() as *mut MIB_IPNETTABLE;

        let ret = GetIpNetTable(Some(table_ptr), &mut bytes_needed, false);
        if ret != 0 {
            panic!("GetIpNetTable failed with code {}", ret);
        }
        let num_entries = (*table_ptr).dwNumEntries as usize;
        let first_row_ptr = (*table_ptr).table.as_ptr();
        let mut map: BTreeMap<u32, Vec<ArpTableEntry>> = BTreeMap::new();
        for i in 0..num_entries {
            let row = &*first_row_ptr.add(i);
            let entry = ArpTableEntry {
                interface_index: row.dwIndex,
                mac_address_len: row.dwPhysAddrLen,
                mac_address: row.bPhysAddr,
                dw_addr: row.dwAddr,
                state: row.Anonymous.Type, // state ARP
            };
            map.entry(entry.interface_index).or_default().push(entry);
        }

        map
    }
}

pub fn attach_arp_entries(
    interfaces: &mut [NetworkInterfaceInfo],
    arp_map: BTreeMap<u32, Vec<ArpTableEntry>>,
) {
    let mut index = BTreeMap::new();

    for iface in interfaces.iter_mut() {
        index.insert(iface.if_index, iface);
    }

    for (if_index, entries) in arp_map {
        if let Some(iface) = index.get_mut(&if_index) {
            iface.arp_entries = entries;
        }
    }
}

fn mac_to_string(bytes: &[u8; 8], len: u32) -> String {
    bytes[..len as usize]
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join("-")
}

fn ipv4_from_u32(addr: u32) -> String {
    std::net::Ipv4Addr::from(u32::from_le(addr)).to_string()
}

pub fn print_interface(interface: &NetworkInterfaceInfo) {
    println!("  {} --- Index {}", interface.name, interface.if_index);
    println!("    Interface Description : {}", interface.description);
    println!("    Interface IPs      : {}", interface.ips.join(", "));

    if !interface.dns_servers.is_empty() {
        println!(
            "    DNS Servers        : {}\n",
            interface.dns_servers.join(", ")
        );
    }

    println!("    Internet Address      Physical Address      Type");

    for e in &interface.arp_entries {
        println!(
            "    {:<22}{:<22}{}",
            ipv4_from_u32(e.dw_addr),
            mac_to_string(&e.mac_address, e.mac_address_len),
            arp_type_to_str(e.state)
        );
    }

    println!();
}

fn main() {
    let mut interfaces = get_network_interfaces();
    let arp_table = get_arp_table();

    attach_arp_entries(&mut interfaces, arp_table);

    for iface in &interfaces {
        print_interface(iface);
    }
}
