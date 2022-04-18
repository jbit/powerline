pub mod broadcom;

mod bridge_info;
mod discover_list;
mod hfid;
mod message;
mod mmeerror;
mod mmtype;
mod mmv;
mod network_info;
mod nmk;
mod station_capabilities;

use crate::{EtherAddr, EtherSocket};
use core::time::Duration;
use log::warn;

pub use bridge_info::BridgeInfo;
pub use bridge_info::*;
pub use discover_list::DiscoverList;
pub use discover_list::*;
pub use hfid::*;
pub use message::*;
pub use mmeerror::*;
pub use mmtype::*;
pub use mmv::*;
pub use network_info::*;
pub use nmk::*;
pub use station_capabilities::StationCapabilities;
pub use station_capabilities::*;

/// Discover devices that respond to `CC_DISCOVER_LIST` broadcast on `socket`
pub fn discover_devices<T: EtherSocket>(
    socket: &mut T,
    mut callback: impl FnMut(EtherAddr, DiscoverList),
) -> Result<(), T::Error> {
    type M = DiscoverListRequest;
    let mut buffer = [0; 1500];
    let request = DiscoverListRequest;
    let bytes = request.encode(&mut buffer);
    socket.sendto(EtherAddr::BROADCAST, bytes)?;

    while let Some((addr, data)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        let msg = UnknownMessage(data);
        if msg.mmv() == M::MMV && msg.mmtype() == M::MMTYPE.cnf() {
            callback(addr, DiscoverList::from(data));
        } else if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            let error = MMEError(data);
            warn!("[{addr:?}] {error:?}");
        } else {
            warn!("[{addr:?}] {msg:?} - Unexpected message");
        }
    }
    Ok(())
}

/// Send a request message and try to receive a reply
pub fn send_request<'a, M: MessageTX<'a>, T: EtherSocket>(
    socket: &mut T,
    buffer: &'a mut [u8; 1500],
    destination: EtherAddr,
    request: M,
) -> Result<Option<M::Response>, T::Error>
where
    M::Response: From<&'a [u8]>,
{
    let bytes = request.encode(buffer);
    socket.sendto(destination, bytes)?;

    let mut result = None;
    while let Some((addr, data)) = socket.recvfrom(buffer, Some(Duration::from_millis(100)))? {
        if destination.is_unicast() && addr != destination {
            continue;
        }
        let msg = UnknownMessage(data);
        if msg.mmv() == M::MMV && msg.mmtype() == M::MMTYPE.cnf() {
            result = Some(M::Response::from(buffer));
            break;
        } else if msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            let error = MMEError(data);
            warn!("[{addr:?}] {error:?}");
        } else {
            warn!("[{addr:?}] {msg:?} - Unexpected message");
        }
    }
    Ok(result)
}
