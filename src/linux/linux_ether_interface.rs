use crate::{linux::*, *};
use libc::{freeifaddrs, getifaddrs};
use libc::{ifaddrs, sockaddr_ll};
use libc::{AF_PACKET, IFF_LOOPBACK, IFF_UP};
use std::io::{Error, Result};
use std::ptr::null_mut;
use std::{convert::TryInto, ffi::CStr, ffi::CString};

pub struct LinuxEtherInterfaceIter {
    first: *mut ifaddrs,
    next: *const ifaddrs,
}
impl LinuxEtherInterfaceIter {
    fn new() -> Result<LinuxEtherInterfaceIter> {
        unsafe {
            let mut first: *mut ifaddrs = null_mut();
            if getifaddrs(&mut first) == -1 {
                return Err(Error::last_os_error());
            }
            Ok(LinuxEtherInterfaceIter { first, next: first })
        }
    }
}
impl Iterator for LinuxEtherInterfaceIter {
    type Item = LinuxInterface;
    fn next(&mut self) -> Option<LinuxInterface> {
        unsafe {
            while let Some(ifaddr) = self.next.as_ref() {
                self.next = ifaddr.ifa_next;
                if let Some(addr) = ifaddr.ifa_addr.as_ref() {
                    if addr.sa_family == AF_PACKET as u16 {
                        return Some(LinuxInterface::new(ifaddr));
                    }
                }
            }
            None
        }
    }
}
impl Drop for LinuxEtherInterfaceIter {
    fn drop(&mut self) {
        unsafe { freeifaddrs(self.first) };
    }
}

pub struct LinuxInterface {
    name: CString,
    flags: i32,
    address: EtherAddr,
}
impl LinuxInterface {
    fn new(ifaddr: &ifaddrs) -> LinuxInterface {
        unsafe {
            let sa = (ifaddr.ifa_addr as *const sockaddr_ll).as_ref().unwrap();
            let address = EtherAddr(sa.sll_addr[..6].try_into().unwrap());
            LinuxInterface {
                name: CStr::from_ptr(ifaddr.ifa_name).to_owned(),
                flags: ifaddr.ifa_flags as i32,
                address,
            }
        }
    }
    pub fn interfaces() -> Result<LinuxEtherInterfaceIter> {
        LinuxEtherInterfaceIter::new()
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
