use clap::App;
use powerline::{homeplug::*, linux::*, *};
use std::time::Duration;
use std::{cmp::max, collections::HashSet};

fn discover_list<T: EtherSocket>(
    socket: &mut T,
    mut callback: impl FnMut(EtherAddr, DiscoverList),
) -> Result<(), T::Error> {
    type M<'a> = DiscoverList<'a>; // TODO: This function should be able to be generic,
    let mut message = [0u8; 60];
    hpav_set_header::<M>(&mut message);
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
    arguments: &[u8],
) -> Result<Option<M>, T::Error> {
    buffer.iter_mut().for_each(|x| *x = 0x00);
    let payload = hpav_set_header::<M>(buffer);
    payload[..arguments.len()].copy_from_slice(arguments);

    let size = max(60, arguments.len() + 8);
    socket.sendto(destination, &buffer[..size])?;

    let mut result = None;

    while let Some((addr, data)) = socket.recvfrom(buffer, Some(Duration::from_millis(100)))? {
        if destination.is_unicast() && addr != destination {
            continue;
        }
        let msg = UnknownMessage(data);
        if msg.mmv() == M::MMV && msg.mmtype() == M::MMTYPE.cnf() {
            result = Some(M::from(buffer));
            break;
        } else if msg.mmtype() == MMType::CM_MME_ERROR.ind() {
            println!("[{:?}] {:?}", addr, MMEError(data));
        } else {
            println!("[{:?}] {:?} - Unexpected message", addr, msg);
        }
    }
    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Power-line communication (HomePlug AV) management utility")
        .get_matches();

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
            let mut oui = OUI::default();

            println!("[{:?}]", addr);
            if let Some(m) = single_message::<StationCapabilities, _>(&mut s, &mut b, addr, &[])? {
                println!("  {:?}", m);
                oui = m.oui();
            }
            if let Some(m) = single_message::<BridgeInfo, _>(&mut s, &mut b, addr, &[])? {
                println!("  {:?}", m);
            }
            if let Some(m) = single_message::<TestMsg, _>(&mut s, &mut b, addr, &[])? {
                println!("  {:?}", m);
            }
            if oui == OUI::BROADCOM {
                let seq = 0x77;
                let mut xs = interface.open(EtherType::MEDIAXTREAM)?;
                if let Some(m) =
                    single_message::<broadcom::GetProperty, _>(&mut xs, &mut b, addr, &[seq, 0x25])?
                {
                    let s = String::from_utf8_lossy(m.records().next().unwrap());
                    println!("  {}", s.trim_end_matches('\0'));
                }
                if let Some(m) =
                    single_message::<broadcom::GetProperty, _>(&mut xs, &mut b, addr, &[seq, 0x26])?
                {
                    let s = String::from_utf8_lossy(m.records().next().unwrap());
                    println!("  {}", s.trim_end_matches('\0'));
                }
            } else {
                if let Some(m) = single_message::<HFID, _>(&mut s, &mut b, addr, &[0x00])? {
                    println!("  {:?}", m);
                }
                if let Some(m) = single_message::<HFID, _>(&mut s, &mut b, addr, &[0x01])? {
                    println!("  {:?}", m);
                }
            }
        }
    }
    Ok(())
}
