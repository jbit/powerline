extern crate std;

use crate::{bsd::*, unix::*, *};
use libc::{ifaddrs, sockaddr_dl};
use libc::{AF_LINK, IFF_LOOPBACK, IFF_UP};
use std::borrow::ToOwned;
use std::ffi::{CStr, CString};
use std::io::{Error, Result};
use std::slice;

pub struct BsdBpfInterface {
    name: CString,
    flags: i32,
    address: EtherAddr,
}
impl BsdBpfInterface {
    pub fn interfaces() -> Result<UnixInterfaceIter<BsdBpfInterface>> {
        UnixInterfaceIter::new()
    }
}
impl UnixInterface for BsdBpfInterface {
    fn new(ifaddr: &ifaddrs) -> Option<BsdBpfInterface> {
        const IFT_ETHER: u8 = 0x06;
        unsafe {
            if ifaddr.ifa_addr.as_ref()?.sa_family != AF_LINK as u8 {
                return None;
            }
            let sa = (ifaddr.ifa_addr as *const sockaddr_dl).as_ref()?;
            if sa.sdl_type != IFT_ETHER || sa.sdl_alen != EtherAddr::SIZE as u8 {
                return None;
            }
            let data_ptr = sa.sdl_data.as_ptr() as *const u8;
            let data_len = (sa.sdl_nlen + sa.sdl_alen + sa.sdl_slen) as usize;
            let data: &[u8] = slice::from_raw_parts(data_ptr, data_len);
            let (n, data) = data.split_at(sa.sdl_nlen as usize); // Name
            let (a, data) = data.split_at(sa.sdl_alen as usize); // Address
            let (s, data) = data.split_at(sa.sdl_slen as usize); // Selector
            let (_, _, _) = (n, s, data);

            Some(BsdBpfInterface {
                name: CStr::from_ptr(ifaddr.ifa_name).to_owned(),
                flags: ifaddr.ifa_flags as i32,
                address: EtherAddr::from_slice(a),
            })
        }
    }
}
impl EtherInterface for BsdBpfInterface {
    type Error = Error;
    type Socket = BsdBpfSocket;
    fn open(&self, ethertype: EtherType) -> Result<BsdBpfSocket> {
        BsdBpfSocket::new(ethertype, &self.name, self.address)
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
impl std::fmt::Debug for BsdBpfInterface {
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
