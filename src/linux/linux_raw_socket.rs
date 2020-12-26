use crate::*;
use libc::{bind, close, if_nametoindex, recvfrom, sendto, socket};
use libc::{c_void, sockaddr, sockaddr_ll};
use libc::{AF_PACKET, ARPHRD_ETHER, SOCK_DGRAM};
use std::convert::TryInto;
use std::ffi::CStr;
use std::io::{Error, ErrorKind, Result};
use std::mem::size_of;
use std::os::unix::io::RawFd;

#[derive(Debug)]
pub struct LinuxRawSocket {
    fd: RawFd,
    ethertype: EtherType,
    ifindex: i32,
}
impl LinuxRawSocket {
    pub(crate) fn new(ethertype: EtherType, inteface: &CStr) -> Result<LinuxRawSocket> {
        unsafe {
            let ifindex = if_nametoindex(inteface.as_ptr()) as i32;
            if ifindex == 0 {
                return Err(Error::last_os_error());
            }

            let fd = socket(AF_PACKET, SOCK_DGRAM, ethertype.as_be_u16() as i32);
            if fd == -1 {
                return Err(Error::last_os_error());
            }

            let sa = sockaddr_ll {
                sll_family: AF_PACKET as u16,
                sll_protocol: ethertype.as_be_u16(),
                sll_ifindex: ifindex,
                sll_hatype: ARPHRD_ETHER,
                sll_pkttype: 0,
                sll_halen: 6,
                sll_addr: [0; 8],
            };
            let addr = &sa as *const _ as *const sockaddr;
            let addrlen = size_of::<sockaddr_ll>() as u32;
            if bind(fd, addr, addrlen) == -1 {
                return Err(Error::last_os_error());
            }

            Ok(LinuxRawSocket {
                fd,
                ethertype,
                ifindex,
            })
        }
    }
}
impl EtherSocket for LinuxRawSocket {
    type Error = Error;
    fn sendto(&mut self, destination: EtherAddr, data: &[u8]) -> Result<()> {
        unsafe {
            let sa = sockaddr_ll {
                sll_family: AF_PACKET as u16,
                sll_protocol: self.ethertype.as_be_u16(),
                sll_ifindex: self.ifindex,
                sll_hatype: ARPHRD_ETHER,
                sll_pkttype: 0,
                sll_halen: destination.len() as u8,
                sll_addr: destination.padded(),
            };
            let buf = data.as_ptr() as *const c_void;
            let len = data.len();
            let addr = &sa as *const _ as *const sockaddr;
            let addrlen = size_of::<sockaddr_ll>() as u32;
            if sendto(self.fd, buf, len, 0, addr, addrlen) == -1 {
                return Err(Error::last_os_error());
            }
            Ok(())
        }
    }
    fn recvfrom<'a>(&mut self, buffer: &'a mut [u8]) -> Result<(EtherAddr, &'a [u8])> {
        unsafe {
            let mut sa = sockaddr_ll {
                sll_family: AF_PACKET as u16,
                sll_protocol: self.ethertype.as_be_u16(),
                sll_ifindex: self.ifindex,
                sll_hatype: ARPHRD_ETHER,
                sll_pkttype: 0,
                sll_halen: 6,
                sll_addr: [0; 8],
            };
            let buf = buffer.as_mut_ptr() as *mut c_void;
            let len = buffer.len();
            let addr = &mut sa as *mut _ as *mut sockaddr;
            let mut addrlen = size_of::<sockaddr_ll>() as u32;
            let size = recvfrom(self.fd, buf, len, 0, addr, &mut addrlen);
            if size == -1 {
                return Err(Error::last_os_error());
            }
            let addr = EtherAddr(sa.sll_addr[..6].try_into().unwrap());
            if size as usize > buffer.len() {
                let msg = format!("Packet from {:?} too big ({}>{})", addr, size, buffer.len());
                return Err(Error::new(ErrorKind::Other, msg));
            }
            let data = &buffer[..size as usize];
            Ok((addr, data))
        }
    }
}
impl Drop for LinuxRawSocket {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}
