use powerline::{homeplug::*, linux::*, *};
use std::{collections::HashSet, time::Duration};

fn discover_list<T: EtherSocket>(
    socket: &mut T,
    mut callback: impl FnMut(DiscoverList<&[u8]>),
) -> Result<(), T::Error> {
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, MMV::HOMEPLUG_AV_1_1, MMType::CC_DISCOVER_LIST);
    socket.sendto(EtherAddr::BROADCAST, &message)?;

    let mut buffer = [0; 1500];
    while let Some((addr, msg)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        let header = Header(msg);

        use MMTypeCode::*;
        match (header.mmv(), header.mmtype().base(), header.mmtype().code()) {
            (_, MMType::CM_MME_ERROR, IND) => {
                println!("{:?}", MMEError(addr, msg));
            }
            (MMV::HOMEPLUG_AV_1_1, MMType::CC_DISCOVER_LIST, CNF) => {
                callback(DiscoverList(addr, msg))
            }
            _ => {
                println!(
                    "[{:?}] {:?}:{:?} - Unexpected message",
                    addr,
                    header.mmv(),
                    header.mmtype()
                );
            }
        }
    }
    Ok(())
}

fn station_capability<T: EtherSocket>(
    socket: &mut T,
    destination: EtherAddr,
) -> Result<(), T::Error> {
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, MMV::HOMEPLUG_AV_1_1, MMType::CM_STA_CAP);
    socket.sendto(destination, &message)?;

    let mut buffer = [0; 1500];
    while let Some((addr, msg)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        if addr != destination {
            continue;
        }
        let header = Header(msg);

        use MMTypeCode::*;
        match (header.mmv(), header.mmtype().base(), header.mmtype().code()) {
            (_, MMType::CM_MME_ERROR, IND) => {
                println!("{:?}", MMEError(addr, msg));
            }
            (MMV::HOMEPLUG_AV_1_1, MMType::CM_STA_CAP, CNF) => {
                let caps = StationCapabilities(addr, msg);
                println!("{:?}", caps);
            }
            _ => {
                println!(
                    "[{:?}] {:?}:{:?} - Unexpected message",
                    addr,
                    header.mmv(),
                    header.mmtype()
                );
            }
        }
    }
    Ok(())
}

fn bridge_info<T: EtherSocket>(socket: &mut T, destination: EtherAddr) -> Result<(), T::Error> {
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, MMV::HOMEPLUG_AV_1_1, MMType::CM_BRG_INFO);
    socket.sendto(destination, &message)?;

    let mut buffer = [0; 1500];
    while let Some((addr, msg)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        if addr != destination {
            continue;
        }
        let header = Header(msg);

        use MMTypeCode::*;
        match (header.mmv(), header.mmtype().base(), header.mmtype().code()) {
            (_, MMType::CM_MME_ERROR, IND) => {
                println!("{:?}", MMEError(addr, msg));
            }
            (MMV::HOMEPLUG_AV_1_1, MMType::CM_BRG_INFO, CNF) => {
                let caps = BridgeInfo(addr, msg);
                print!("{:?}", caps);
            }
            _ => {
                println!(
                    "[{:?}] {:?}:{:?} - Unexpected message",
                    addr,
                    header.mmv(),
                    header.mmtype()
                );
            }
        }
    }
    Ok(())
}

fn main() {
    for interface in LinuxInterface::interfaces().unwrap() {
        if !interface.is_up() || interface.is_loopback() {
            continue;
        }
        println!();
        println!("Interface {:?}", interface);
        println!("---------");
        let mut socket = interface.open(EtherType::HOMEPLUG_AV).unwrap();

        let mut all_stations: HashSet<EtherAddr> = HashSet::new();

        discover_list(&mut socket, |list| {
            print!("{:?}", list);
            all_stations.insert(list.0);
            for station in list.stations() {
                all_stations.insert(station.addr());
            }
        })
        .unwrap();

        // Try to query the capabilities of all stations, not just ones that replied to above discover messages
        for addr in all_stations {
            station_capability(&mut socket, addr).unwrap();
            bridge_info(&mut socket, addr).unwrap();
            println!();
        }
    }
}
