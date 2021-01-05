use clap::{App, Arg};
use powerline::{homeplug::*, *};
use std::cmp::max;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::FromStr;
use std::time::Duration;

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

fn scan_on_interface<T: EtherInterface>(interface: T) -> Result<(), T::Error> {
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
        if oui == OUI::BROADCOM {
            let seq = 0x77;
            let mut xs = interface.open(EtherType::MEDIAXTREAM)?;
            let a = &[seq, broadcom::Property::NAME_A1.0];
            if let Some(m) = single_message::<broadcom::GetProperty, _>(&mut xs, &mut b, addr, a)? {
                let s = String::from_utf8_lossy(m.records().next().unwrap());
                println!("  Name: {}", s.trim_end_matches('\0'));
            }
        } else {
            let a = &[HFIDRequest::GET_USR.0];
            if let Some(m) = single_message::<HFID, _>(&mut s, &mut b, addr, a)? {
                println!("  Name: {}", m.hfid());
            }
        }
    }
    Ok(())
}

fn scan<T: EtherInterface>(
    interfaces: impl Iterator<Item = T>,
    mut filter: Option<HashSet<String>>,
) -> Result<(), T::Error> {
    for interface in interfaces {
        let selected = filter.as_mut().map_or_else(
            || interface.is_up() && !interface.is_loopback(),
            |set| set.remove(interface.name()),
        );
        if selected {
            println!();
            println!("Interface {:?}", interface);
            println!("---------");
            if let Err(e) = scan_on_interface(interface) {
                println!("Failed: {:?}", e);
            }
        }
    }

    if let Some(filter) = filter {
        if !filter.is_empty() {
            println!();
            println!("Unknown interfaces specified: {:?}", filter);
        }
    }
    Ok(())
}

fn find_device<T: EtherInterface>(
    interfaces: impl Iterator<Item = T>,
    mut filter: Option<HashSet<String>>,
    addr: EtherAddr,
) -> Result<Option<(T, OUI)>, T::Error> {
    for interface in interfaces {
        let selected = filter.as_mut().map_or_else(
            || interface.is_up() && !interface.is_loopback(),
            |set| set.remove(interface.name()),
        );
        if selected {
            let mut s = interface.open(EtherType::HOMEPLUG_AV)?;
            let mut b = [0; 1500];
            if let Some(m) = single_message::<StationCapabilities, _>(&mut s, &mut b, addr, &[])? {
                return Ok(Some((interface, m.oui())));
            }
        }
    }

    if let Some(filter) = filter {
        if !filter.is_empty() {
            println!();
            println!("Unknown interfaces specified: {:?}", filter);
        }
    }
    Ok(None)
}

fn set_name<T: EtherInterface>(
    interface: T,
    addr: EtherAddr,
    oui: OUI,
    name: &str,
) -> Result<(), T::Error> {
    let mut b = [0; 1500];
    let name = name.trim().as_bytes();

    if oui == OUI::BROADCOM {
        // Broadcom HPAV2 devices don't support the standard HomePlug AV HFID commands
        let mut s = interface.open(EtherType::MEDIAXTREAM)?;

        let seq = 0x80;
        let property = broadcom::Property::NAME_A1.0;
        let count = 1;
        let record_size = 64u16;
        let mut a = [0u8; 70];
        a[0] = seq;
        a[1] = property;
        a[2] = 0x00; // ????
        a[3] = count;
        a[4] = record_size.to_le_bytes()[0];
        a[5] = record_size.to_le_bytes()[1];
        a[6..]
            .iter_mut()
            .zip(name)
            .for_each(|(dest, src)| *dest = *src);

        if let Some(m) = single_message::<broadcom::SetProperty, _>(&mut s, &mut b, addr, &a)? {
            println!("  {:?}", m);
        }
    } else {
        let mut s = interface.open(EtherType::HOMEPLUG_AV)?;

        let mut a = [0u8; 64];
        a[0] = HFIDRequest::SET_USR.0;
        a[1..]
            .iter_mut()
            .zip(name)
            .for_each(|(dest, src)| *dest = *src);

        if let Some(m) = single_message::<HFID, _>(&mut s, &mut b, addr, &a)? {
            println!("  {:?}", m);
        }
    }

    Ok(())
}

fn valid_etheraddr(s: String) -> Result<(), String> {
    EtherAddr::from_str(&s)
        .map(|_| ())
        .map_err(|_| format!("Invalid MAC address ('{}')", s))
}

fn main() {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Power-line communication (HomePlug AV) management utility")
        .arg(
            Arg::with_name("interfaces")
                .long("interfaces")
                .help("Select the interface(s) to discover with")
                .multiple(true)
                .use_delimiter(true)
                .global(true),
        )
        .subcommand(App::new("scan").about("Discover and list devices"))
        .subcommand(
            App::new("find")
                .about("Find which interface a specific device is reachable")
                .arg(
                    Arg::with_name("device")
                        .required(true)
                        .validator(valid_etheraddr),
                ),
        )
        .subcommand(
            App::new("set-name")
                .about("Set the name of a device")
                .args(&[
                    Arg::with_name("device")
                        .required(true)
                        .validator(valid_etheraddr),
                    Arg::with_name("name").required(true),
                ]),
        )
        .get_matches();

    let filter = matches
        .values_of_lossy("interfaces")
        .map(HashSet::from_iter);

    let interfaces = platform_interfaces().unwrap();

    match matches.subcommand() {
        ("find", Some(args)) => {
            let addr = EtherAddr::from_str(&args.value_of_lossy("device").unwrap()).unwrap();
            if let Some((interface, _)) = find_device(interfaces, filter, addr).unwrap() {
                println!("{:?}: Found on {}", addr, interface.name());
            } else {
                println!("{:?}: Not found", addr);
            }
        }
        ("set-name", Some(args)) => {
            let addr = EtherAddr::from_str(&args.value_of_lossy("device").unwrap()).unwrap();
            let name = args.value_of_lossy("name").unwrap();
            if let Some((interface, oui)) = find_device(interfaces, filter, addr).unwrap() {
                set_name(interface, addr, oui, &name).unwrap();
            } else {
                println!("{:?}: Not found", addr);
            }
        }
        ("scan", _) | ("", _) => {
            scan(interfaces, filter).unwrap();
        }
        _ => panic!(),
    }
}
