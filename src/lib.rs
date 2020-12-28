#![no_std]
#[cfg(any(target_os = "linux", target_os = "android"))]
pub mod linux;

pub mod homeplug;

mod ether_addr;
mod ether_type;
mod oui;

use core::time::Duration;

pub use ether_addr::*;
pub use ether_type::*;
pub use oui::*;

pub trait EtherSocket {
    type Error: core::fmt::Debug;
    fn sendto(&mut self, destination: EtherAddr, data: &[u8]) -> Result<(), Self::Error>;
    fn recvfrom<'a>(
        &mut self,
        buffer: &'a mut [u8],
        timeout: Option<Duration>,
    ) -> Result<Option<(EtherAddr, &'a [u8])>, Self::Error>;
}

pub trait EtherInterface: core::fmt::Debug {
    type Error: core::fmt::Debug;
    type Socket: EtherSocket;
    fn open(&self, ethertype: EtherType) -> Result<Self::Socket, Self::Error>;
    fn name(&self) -> &str;
    fn address(&self) -> EtherAddr;
    fn is_up(&self) -> bool;
    fn is_loopback(&self) -> bool;
}
