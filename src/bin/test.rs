use powerline::{homeplug::*, linux::*, *};
use std::collections::HashSet;
use std::time::Duration;

fn discover_list<T: EtherSocket>(
    socket: &mut T,
    mut callback: impl FnMut(EtherAddr, DiscoverList),
) -> Result<(), T::Error> {
    type M<'a> = DiscoverList<'a>; // TODO: This function should be able to be generic,
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, M::MMV, M::MMTYPE);
    socket.sendto(EtherAddr::BROADCAST, &message)?;

    let mut buffer = [0; 1500];
    while let Some((addr, data)) = socket.recvfrom(&mut buffer, Some(Duration::from_millis(100)))? {
        let msg = UnknownMessage(data);
        if msg.mmv() == M::MMV && msg.mmtype() == M::MMTYPE.cnf() {
            callback(addr, M::from(data));
        } else if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            println!("[{:?}] {:?}", addr, MMEError(data));
        } else {
            println!("[{:?}] {:?} - Unexpected message", addr, msg);
        }
    }
    Ok(())
}

fn single_message<'a, M: Message + From<&'a [u8]>, T: EtherSocket>(
    socket: &mut T,
    buffer: &'a mut [u8; 1500],
    destination: EtherAddr,
) -> Result<Option<M>, T::Error> {
    let mut message = [0u8; 60];
    hpav_set_header(&mut message, M::MMV, M::MMTYPE);
    socket.sendto(destination, &message)?;

    let mut result = None;

    while let Some((addr, data)) = socket.recvfrom(buffer, Some(Duration::from_millis(100)))? {
        if destination.is_unicast() && addr != destination {
            continue;
        }
        let msg = UnknownMessage(data);
        if msg.mmv() == M::MMV && msg.mmtype() == M::MMTYPE.cnf() {
            result = Some(M::from(buffer));
            break;
        } else if msg.mmv() == MMV::HOMEPLUG_AV_1_1 && msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            println!("[{:?}] {:?}", addr, MMEError(data));
        } else {
            println!("[{:?}] {:?} - Unexpected message", addr, msg);
        }
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for interface in LinuxInterface::interfaces()? {
        if !interface.is_up() || interface.is_loopback() {
            continue;
        }
        println!();
        println!("Interface {:?}", interface);
        println!("---------");
        let mut s = interface.open(EtherType::HOMEPLUG_AV)?;

        let mut all_stations: HashSet<EtherAddr> = HashSet::new();

        discover_list(&mut s, |addr, msg| {
            println!("[{:?}] {:?}", addr, msg);
            all_stations.insert(addr);
            for station in msg.stations() {
                all_stations.insert(station.addr());
            }
        })?;

        // Try to query all stations, not just ones that replied directly to above discover messages
        for addr in all_stations {
            let mut b = [0; 1500];
            println!("[{:?}]", addr);
            if let Some(m) = single_message::<StationCapabilities, _>(&mut s, &mut b, addr)? {
                println!("  {:?}", m);
            }
            if let Some(m) = single_message::<BridgeInfo, _>(&mut s, &mut b, addr)? {
                println!("  {:?}", m);
            }
        }
    }
    Ok(())
}
