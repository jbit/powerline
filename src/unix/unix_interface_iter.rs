extern crate std;

use crate::*;
use core::{marker::PhantomData, ptr::null_mut};
use libc::ifaddrs;
use libc::{freeifaddrs, getifaddrs};
use std::io::{Error, Result};

pub trait UnixInterface: EtherInterface + Sized {
    fn new(addr: &ifaddrs) -> Option<Self>;
}

pub struct UnixInterfaceIter<T: UnixInterface> {
    first: *mut ifaddrs,
    next: *const ifaddrs,
    phantom: PhantomData<T>,
}
impl<T: UnixInterface> UnixInterfaceIter<T> {
    pub(crate) fn new() -> Result<UnixInterfaceIter<T>> {
        unsafe {
            let mut first: *mut ifaddrs = null_mut();
            if getifaddrs(&mut first) == -1 {
                return Err(Error::last_os_error());
            }
            Ok(UnixInterfaceIter {
                first,
                next: first,
                phantom: PhantomData,
            })
        }
    }
}
impl<T: UnixInterface> Iterator for UnixInterfaceIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        unsafe {
            while let Some(ifaddr) = self.next.as_ref() {
                self.next = ifaddr.ifa_next;
                if let Some(interface) = T::new(ifaddr) {
                    return Some(interface);
                }
            }
            None
        }
    }
}
impl<T: UnixInterface> Drop for UnixInterfaceIter<T> {
    fn drop(&mut self) {
        unsafe { freeifaddrs(self.first) };
    }
}
