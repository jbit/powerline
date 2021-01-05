extern crate std;

use crate::{linux::*, unix::*, *};
use core::convert::TryInto;
use libc::{ifaddrs, sockaddr_ll};
use libc::{AF_PACKET, IFF_LOOPBACK, IFF_UP};
use std::borrow::ToOwned;
use std::ffi::{CStr, CString};
use std::io::{Error, Result};

pub struct LinuxInterface {
    name: CString,
    flags: i32,
    address: EtherAddr,
}
impl LinuxInterface {
    pub fn interfaces() -> Result<UnixInterfaceIter<LinuxInterface>> {
        UnixInterfaceIter::new()
    }
}
impl UnixInterface for LinuxInterface {
    fn new(ifaddr: &ifaddrs) -> Option<LinuxInterface> {
        unsafe {
            if ifaddr.ifa_addr.as_ref()?.sa_family != AF_PACKET as u16 {
                return None;
            }
            let sa = (ifaddr.ifa_addr as *const sockaddr_ll).as_ref().unwrap();
            let address = EtherAddr(sa.sll_addr[..6].try_into().unwrap());
            Some(LinuxInterface {
                name: CStr::from_ptr(ifaddr.ifa_name).to_owned(),
                flags: ifaddr.ifa_flags as i32,
                address,
            })
        }
    }
}
impl EtherInterface for LinuxInterface {
    type Error = Error;
    type Socket = LinuxRawSocket;
    fn open(&self, ethertype: EtherType) -> Result<LinuxRawSocket> {
        LinuxRawSocket::new(ethertype, &self.name)
    }
    fn name(&self) -> &str {
        self.name.to_str().unwrap_or("<INVALID>")
    }
    fn address(&self) -> EtherAddr {
        self.address
    }
    fn is_up(&self) -> bool {
        self.flags & IFF_UP != 0
    }
    fn is_loopback(&self) -> bool {
        self.flags & IFF_LOOPBACK != 0
    }
}
impl std::fmt::Debug for LinuxInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let status = if self.is_up() { "up" } else { "down" };
        write!(f, "{:?} - {} ({}", self.address(), self.name(), status)?;

        if self.is_loopback() {
            write!(f, " loopback")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
