//! Host lookups for `SockGetHostByName` and `SockGetHostByAddr`, as glibc's
//! legacy `gethostbyname` and `gethostbyaddr` answer them for `AF_INET`
//! (`extensions-behaviour.md`, "Host stems"). Those calls are not reachable
//! without `unsafe`, so the two sources the host's `hosts: files dns` names
//! are read here in their order: `/etc/hosts`, parsed as glibc's files
//! backend parses it, with `multi` from `/etc/host.conf`, and then the
//! resolver through `getaddrinfo` and `getnameinfo` (`dns-lookup`), which
//! give the canonical name and the addresses but no aliases.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4};

use super::inet_aton;

/// What a lookup found.
pub(super) struct Host {
    pub(super) name: Vec<u8>,
    pub(super) aliases: Vec<Vec<u8>>,
    pub(super) addresses: Vec<Ipv4Addr>,
}

/// One line of the hosts file: its address, as an `AF_INET` lookup reads
/// it, and its names, the official one first.
struct Entry {
    address: Ipv4Addr,
    names: Vec<Vec<u8>>,
}

/// An address field as glibc's files backend reads it for `AF_INET`: a
/// dotted quad, or an IPv6 address that is IPv4-mapped (its last four bytes)
/// or the loopback `::1` (127.0.0.1). `None` skips the line.
fn entry_address(text: &str) -> Option<Ipv4Addr> {
    if let Ok(v4) = text.parse::<Ipv4Addr>() {
        return Some(v4);
    }
    let v6 = text.parse::<Ipv6Addr>().ok()?;
    if let Some(v4) = v6.to_ipv4_mapped() {
        return Some(v4);
    }
    (v6 == Ipv6Addr::LOCALHOST).then_some(Ipv4Addr::LOCALHOST)
}

/// The hosts file's entries, in file order.
fn entries() -> Vec<Entry> {
    let Ok(text) = std::fs::read("/etc/hosts") else {
        return Vec::new();
    };
    text.split(|byte| *byte == b'\n')
        .filter_map(|line| {
            let line = match line.iter().position(|byte| *byte == b'#') {
                Some(comment) => &line[..comment],
                None => line,
            };
            let mut fields = line
                .split(u8::is_ascii_whitespace)
                .filter(|field| !field.is_empty());
            let address = std::str::from_utf8(fields.next()?).ok()?;
            let address = entry_address(address)?;
            let names: Vec<Vec<u8>> = fields.map(<[u8]>::to_vec).collect();
            (!names.is_empty()).then_some(Entry { address, names })
        })
        .collect()
}

/// Whether `/etc/host.conf` (or `RESOLV_MULTI`) turns on `multi`, which
/// makes a name lookup gather every matching hosts-file line.
fn multi() -> bool {
    if let Some(value) = std::env::var_os("RESOLV_MULTI") {
        return value.eq_ignore_ascii_case("on");
    }
    let Ok(text) = std::fs::read_to_string("/etc/host.conf") else {
        return false;
    };
    let mut on = false;
    for line in text.lines() {
        let mut words = line.split_whitespace();
        if words
            .next()
            .is_some_and(|word| word.eq_ignore_ascii_case("multi"))
        {
            on = words
                .next()
                .is_some_and(|word| word.eq_ignore_ascii_case("on"));
        }
    }
    on
}

/// `gethostbyname(name)` for `AF_INET`, `None` where it finds nothing.
pub(super) fn by_name(name: &[u8]) -> Option<Host> {
    // glibc's `__nss_hostname_digits_dots`: a name of digits and dots, not
    // ending in a dot, is an address or nothing.
    if name.first().is_some_and(u8::is_ascii_digit)
        && name
            .iter()
            .all(|byte| byte.is_ascii_digit() || *byte == b'.')
        && name.last() != Some(&b'.')
    {
        return inet_aton(name).map(|address| Host {
            name: name.to_vec(),
            aliases: Vec::new(),
            addresses: vec![address],
        });
    }
    if let Some(found) = by_name_in_files(name) {
        return Some(found);
    }
    // A name `inet_aton` reads that the digits-and-dots test let through
    // (`0x7f.1`) is a name to the resolver, which `getaddrinfo` would read as
    // an address instead.
    if name.is_empty() || inet_aton(name).is_some() {
        return None;
    }
    by_name_in_dns(name)
}

fn by_name_in_files(name: &[u8]) -> Option<Host> {
    let matches = |entry: &Entry| {
        entry
            .names
            .iter()
            .any(|known| known.eq_ignore_ascii_case(name))
    };
    let mut found = entries().into_iter().filter(matches);
    let first = found.next()?;
    let mut names = first.names.into_iter();
    let mut host = Host {
        name: names.next().unwrap_or_default(),
        aliases: names.collect(),
        addresses: vec![first.address],
    };
    if multi() {
        for entry in found {
            let mut names = entry.names.into_iter();
            if let Some(official) = names.next()
                && official != host.name
            {
                host.aliases.push(official);
            }
            host.aliases.extend(names);
            host.addresses.push(entry.address);
        }
    }
    Some(host)
}

fn by_name_in_dns(name: &[u8]) -> Option<Host> {
    let name = std::str::from_utf8(name).ok()?;
    let hints = dns_lookup::AddrInfoHints {
        flags: libc::AI_CANONNAME,
        address: dns_lookup::AddrFamily::Inet.into(),
        socktype: dns_lookup::SockType::Stream.into(),
        protocol: 0,
    };
    let answers = dns_lookup::getaddrinfo(Some(name), None, Some(hints)).ok()?;
    let mut canonical = None;
    let mut addresses = Vec::new();
    for info in answers.flatten() {
        if canonical.is_none() {
            canonical = info.canonname.clone();
        }
        if let SocketAddr::V4(v4) = info.sockaddr {
            addresses.push(*v4.ip());
        }
    }
    if addresses.is_empty() {
        return None;
    }
    Some(Host {
        name: canonical.unwrap_or_else(|| name.to_owned()).into_bytes(),
        aliases: Vec::new(),
        addresses,
    })
}

/// `gethostbyaddr(address, 4, AF_INET)`, `None` where it finds nothing.
pub(super) fn by_addr(address: Ipv4Addr) -> Option<Host> {
    if let Some(entry) = entries().into_iter().find(|entry| entry.address == address) {
        let mut names = entry.names.into_iter();
        return Some(Host {
            name: names.next().unwrap_or_default(),
            aliases: names.collect(),
            addresses: vec![address],
        });
    }
    let socket = SocketAddr::V4(SocketAddrV4::new(address, 0));
    let (name, _) = dns_lookup::getnameinfo(&socket, libc::NI_NAMEREQD).ok()?;
    if name.parse::<IpAddr>().is_ok() {
        return None;
    }
    Some(Host {
        name: name.into_bytes(),
        aliases: Vec::new(),
        addresses: vec![address],
    })
}
