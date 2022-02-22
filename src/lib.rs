#![no_std]

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "linux",
    target_os = "android"
))]
pub mod unix;

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub mod bsd;

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

pub trait EtherSocket: core::fmt::Debug + Send + Sync + 'static {
    type Error: core::fmt::Debug + core::fmt::Display + Send + Sync + 'static;
    fn sendto(&mut self, destination: EtherAddr, data: &[u8]) -> Result<(), Self::Error>;
    fn recvfrom<'a>(
        &mut self,
        buffer: &'a mut [u8],
        timeout: Option<Duration>,
    ) -> Result<Option<(EtherAddr, &'a [u8])>, Self::Error>;
}

pub trait EtherInterface: core::fmt::Debug + Send + Sync + 'static {
    type Error: core::fmt::Debug + core::fmt::Display + Send + Sync + 'static;
    type Socket: EtherSocket<Error = Self::Error>;
    fn open(&self, ethertype: EtherType) -> Result<Self::Socket, Self::Error>;
    fn name(&self) -> &str;
    fn address(&self) -> EtherAddr;
    fn is_up(&self) -> bool;
    fn is_loopback(&self) -> bool;
}

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
pub fn platform_interfaces(
) -> Result<impl Iterator<Item = impl EtherInterface>, impl core::fmt::Debug> {
    bsd::BsdBpfInterface::interfaces()
}

#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn platform_interfaces(
) -> Result<impl Iterator<Item = impl EtherInterface>, impl core::fmt::Debug> {
    linux::LinuxInterface::interfaces()
}
