extern crate std;

use crate::*;
use core::convert::TryInto;
use core::mem::size_of;
use libc::{bind, close, if_nametoindex, recvfrom, sendto, setsockopt, socket};
use libc::{c_void, sockaddr, sockaddr_ll, suseconds_t, time_t, timeval};
use libc::{AF_PACKET, ARPHRD_ETHER, SOCK_DGRAM, SOL_SOCKET, SO_RCVTIMEO};
use std::ffi::CStr;
use std::format;
use std::io::{Error, ErrorKind, Result};
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
            let mut socket = LinuxRawSocket {
                fd,
                ethertype,
                ifindex,
            };
            socket.bind()?;

            Ok(socket)
        }
    }
    fn bind(&mut self) -> Result<()> {
        let sa = sockaddr_ll {
            sll_family: AF_PACKET as u16,
            sll_protocol: self.ethertype.as_be_u16(),
            sll_ifindex: self.ifindex,
            sll_hatype: ARPHRD_ETHER,
            sll_pkttype: 0,
            sll_halen: 6,
            sll_addr: [0; 8],
        };
        let addr = &sa as *const _ as *const sockaddr;
        let addrlen = size_of::<sockaddr_ll>() as u32;
        if unsafe { bind(self.fd, addr, addrlen) == -1 } {
            return Err(Error::last_os_error());
        }
        Ok(())
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
    fn recvfrom<'a>(
        &mut self,
        buffer: &'a mut [u8],
        timeout: Option<Duration>,
    ) -> Result<Option<(EtherAddr, &'a [u8])>> {
        unsafe {
            let tv = if let Some(timeout) = timeout {
                timeval {
                    tv_sec: timeout.as_secs() as time_t,
                    tv_usec: timeout.subsec_micros() as suseconds_t,
                }
            } else {
                timeval {
                    tv_sec: 0,
                    tv_usec: 0,
                }
            };
            setsockopt(
                self.fd,
                SOL_SOCKET,
                SO_RCVTIMEO,
                &tv as *const _ as *const c_void,
                size_of::<timeval>() as u32,
            );

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
                let e = Error::last_os_error();
                if e.kind() == ErrorKind::WouldBlock {
                    return Ok(None);
                } else {
                    return Err(e);
                }
            }
            let addr = EtherAddr(sa.sll_addr[..6].try_into().unwrap());
            if size as usize > buffer.len() {
                let msg = format!("Packet from {:?} too big ({}>{})", addr, size, buffer.len());
                return Err(Error::new(ErrorKind::Other, msg));
            }
            let data = &buffer[..size as usize];
            Ok(Some((addr, data)))
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
