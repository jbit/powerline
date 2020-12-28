use powerline::{homeplug::*, linux::*, *};
use std::{collections::HashSet, time::Duration};

fn discover_list<T: EtherSocket>(
    socket: &mut T,
    mut callback: impl FnMut(EtherAddr, DiscoverList),
) -> Result<(), T::Error> {
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, MMV::HOMEPLUG_AV_1_1, MMType::CC_DISCOVER_LIST);
    socket.sendto(EtherAddr::BROADCAST, &message)?;

    let mut buffer = [0; 1500];
    while let Some((addr, data)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        let msg = UnknownMessage(data);
        if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CC_DISCOVER_LIST.cnf() {
            callback(addr, DiscoverList(data));
        } else if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            println!("[{:?}] {:?}", addr, MMEError(data));
        } else {
            println!("[{:?}] {:?} - Unexpected message", addr, msg);
        }
    }
    Ok(())
}

fn station_capability<T: EtherSocket>(
    socket: &mut T,
    destination: EtherAddr,
    mut callback: impl FnMut(EtherAddr, StationCapabilities),
) -> Result<(), T::Error> {
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, MMV::HOMEPLUG_AV_1_1, MMType::CM_STA_CAP);
    socket.sendto(destination, &message)?;

    let mut buffer = [0; 1500];
    while let Some((addr, data)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        if addr != destination {
            continue;
        }
        let msg = UnknownMessage(data);
        if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_STA_CAP.cnf() {
            callback(addr, StationCapabilities(data));
        } else if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            println!("[{:?}] {:?}", addr, MMEError(data));
        } else {
            println!("[{:?}] {:?} - Unexpected message", addr, msg);
        }
    }
    Ok(())
}

fn bridge_info<T: EtherSocket>(
    socket: &mut T,
    destination: EtherAddr,
    mut callback: impl FnMut(EtherAddr, BridgeInfo),
) -> Result<(), T::Error> {
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, MMV::HOMEPLUG_AV_1_1, MMType::CM_BRG_INFO);
    socket.sendto(destination, &message)?;

    let mut buffer = [0; 1500];
    while let Some((addr, data)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        if addr != destination {
            continue;
        }
        let msg = UnknownMessage(data);
        if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_BRG_INFO.cnf() {
            callback(addr, BridgeInfo(data));
        } else if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            println!("[{:?}] {:?}", addr, MMEError(data));
        } else {
            println!("[{:?}] {:?} - Unexpected message", addr, msg);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for interface in LinuxInterface::interfaces()? {
        if !interface.is_up() || interface.is_loopback() {
            continue;
        }
        println!();
        println!("Interface {:?}", interface);
        println!("---------");
        let mut socket = interface.open(EtherType::HOMEPLUG_AV)?;

        let mut all_stations: HashSet<EtherAddr> = HashSet::new();

        discover_list(&mut socket, |addr, msg| {
            println!("[{:?}] {:?}", addr, msg);
            all_stations.insert(addr);
            for station in msg.stations() {
                all_stations.insert(station.addr());
            }
        })?;

        // Try to query the capabilities of all stations, not just ones that replied to above discover messages
        for addr in all_stations {
            println!("[{:?}]", addr);

            station_capability(&mut socket, addr, |_, msg| {
                println!("  {:?}", msg);
            })?;
            bridge_info(&mut socket, addr, |_, msg| {
                println!("  {:?}", msg);
            })?;
        }
    }
    Ok(())
}
