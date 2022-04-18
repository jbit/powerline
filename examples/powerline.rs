use clap::{App, Arg};
use log::{debug, info, warn};
use powerline::{homeplug::*, *};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::FromStr;

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .trim_end_matches('\0')
        .to_string()
}

fn scan_on_interface<T: EtherInterface>(interface: &T) -> Result<(), T::Error> {
    let mut s = interface.open(EtherType::HOMEPLUG_AV)?;

    let mut all_stations: HashSet<EtherAddr> = HashSet::new();

    discover_devices(&mut s, |addr, msg| {
        info!("[{addr:?}] {msg:?}");
        all_stations.insert(addr);
        for station in msg.stations() {
            all_stations.insert(station.addr());
        }
    })?;

    info!("Discovered {} stations", all_stations.len());

    // Try to query all stations, not just ones that replied directly to above discover messages
    for addr in all_stations {
        let mut b = [0; 1500];
        let mut oui = OUI::default();
        let mut version = Default::default();
        let mut bridged = 0;

        info!("");
        info!("[{addr:?}]");
        if let Some(m) = send_request(&mut s, &mut b, addr, StationCapabilitiesRequest)? {
            info!("  {m:?}");
            oui = m.oui();
            version = m.version();
        }
        if let Some(m) = send_request(&mut s, &mut b, addr, BridgeInfoRequest)? {
            info!("  {m:?}");
            bridged = m.destinations().count();
        }
        let mut name = None;
        if oui == OUI::BROADCOM {
            let mut seq = 0x77;
            let mut xs = interface.open(EtherType::MEDIAXTREAM)?;
            let request = broadcom::GetPropertyRequest {
                seq,
                property: broadcom::Property::HFID_USER,
            };
            if let Some(m) = send_request(&mut xs, &mut b, addr, request)? {
                name = Some(bytes_to_string(m.first().unwrap()));
            }
            seq += 1;
            let request = broadcom::GetPropertyRequest {
                seq,
                property: broadcom::Property::NAME_A0,
            };
            if let Some(m) = send_request(&mut xs, &mut b, addr, request)? {
                let firmware_name = bytes_to_string(m.first().unwrap());
                info!("  Firmware({firmware_name})");
            }
            seq += 1;
            let request = broadcom::GetPropertyRequest {
                seq,
                property: broadcom::Property::NAME_B0,
            };
            if let Some(m) = send_request(&mut xs, &mut b, addr, request)? {
                let hardware_name = bytes_to_string(m.first().unwrap());
                info!("  Hardware({hardware_name})");
            }
        } else {
            if let Some(m) = send_request(&mut s, &mut b, addr, HFIDRequest::GetUsr)? {
                name = Some(m.hfid().to_string());
            }
            if let Some(m) = send_request(&mut s, &mut b, addr, HFIDRequest::GetMfg)? {
                let hardware_name = m.hfid().to_string();
                info!("  Hardware({hardware_name})");
            }
        };
        let name = name.unwrap_or_default();
        println!("{interface}: [{addr:?}] {version:?} {oui:?} {bridged}Ethers '{name}'",);
    }
    Ok(())
}

fn scan<T: EtherInterface>(
    interfaces: impl Iterator<Item = T>,
    mut filter: Option<HashSet<String>>,
) {
    for interface in interfaces {
        let selected = filter.as_mut().map_or_else(
            || interface.is_up() && !interface.is_loopback(),
            |set| set.remove(interface.name()),
        );
        if selected {
            let header = format!("Scanning Interface {interface:?}");
            info!("");
            info!("{header}");
            info!("{}", "-".repeat(header.len()));
            if let Err(err) = scan_on_interface(&interface) {
                info!("{interface}: Failed to scan ({err})");
            }
        } else {
            info!("{interface}: Skipped Interface");
        }
    }

    if let Some(filter) = filter {
        if !filter.is_empty() {
            warn!("");
            warn!("Unknown interfaces specified: {filter:?}");
        }
    }
}

fn find_device<T: EtherInterface>(
    interfaces: impl Iterator<Item = T>,
    mut filter: Option<HashSet<String>>,
    addr: EtherAddr,
) -> Result<Option<(T, OUI)>, T::Error> {
    debug!("{addr:?} searching for interface");
    for interface in interfaces {
        let selected = filter.as_mut().map_or_else(
            || interface.is_up() && !interface.is_loopback(),
            |set| set.remove(interface.name()),
        );
        if selected {
            let mut s = interface.open(EtherType::HOMEPLUG_AV)?;
            let mut b = [0; 1500];
            if let Some(m) = send_request(&mut s, &mut b, addr, StationCapabilitiesRequest)? {
                debug!("{addr:?} found on {interface}");
                return Ok(Some((interface, m.oui())));
            }
        }
    }

    if let Some(filter) = filter {
        if !filter.is_empty() {
            warn!("");
            warn!("Unknown interfaces specified: {filter:?}");
        }
    }

    debug!("{addr:?} not found on any selected interface");

    Ok(None)
}

fn dump<T: EtherInterface>(interfaces: impl Iterator<Item = T>) {
    let mut threads = vec![];
    for interface in interfaces {
        match interface.open(EtherType::HOMEPLUG_AV) {
            Ok(mut socket) => threads.push(std::thread::spawn(move || {
                debug!("Listening for messages on {interface:?}");
                let mut buffer = [0; 1500];
                while let Some((addr, data)) = socket.recvfrom(&mut buffer, None).unwrap() {
                    let msg = UnknownMessage(data);
                    println!("{interface:w$} [{addr:?}] {msg:?}", w = 16);
                }
            })),
            Err(err) => {
                warn!("Failed to listen on '{interface:?}': {err}");
            }
        }
    }
    for t in threads {
        t.join().unwrap();
    }
}

fn set_name<T: EtherInterface>(
    interface: T,
    addr: EtherAddr,
    oui: OUI,
    name: &str,
) -> Result<(), T::Error> {
    let mut b = [0; 1500];
    let mut hfid = [0u8; 64];
    hfid.iter_mut()
        .zip(name.trim().as_bytes())
        .for_each(|(dest, src)| *dest = *src);

    if oui == OUI::BROADCOM {
        // Broadcom HPAV2 devices don't support the standard HomePlug AV HFID commands
        let mut s = interface.open(EtherType::MEDIAXTREAM)?;
        let req = broadcom::SetPropertyRequest {
            seq: 0x80,
            property: broadcom::Property::HFID_USER,
            data: hfid,
        };
        if let Some(m) = send_request(&mut s, &mut b, addr, req)? {
            println!("  {m:?}");
        }
    } else {
        let mut s = interface.open(EtherType::HOMEPLUG_AV)?;
        let req = HFIDRequest::SetUsr { hfid };
        if let Some(m) = send_request(&mut s, &mut b, addr, req)? {
            println!("  {m:?}");
        }
    }

    Ok(())
}

fn valid_etheraddr(s: String) -> Result<(), String> {
    EtherAddr::from_str(&s)
        .map(|_| ())
        .map_err(|_| format!("Invalid MAC address ('{s}')"))
}

struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{}", record.args());
        }
    }
    fn flush(&self) {}
}

fn main() {
    log::set_logger(&Logger).unwrap();

    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Power-line communication (HomePlug AV) management utility")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .help("Increase verbosity")
                .global(true)
                .multiple(true)
                .takes_value(false),
        )
        .arg(
            Arg::with_name("interfaces")
                .long("interface")
                .help("Select the interface(s) to discover with")
                .multiple(true)
                .number_of_values(1)
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
        .subcommand(App::new("dump").about("Dump all messages"))
        .get_matches();

    match matches.occurrences_of("verbose") {
        0 => log::set_max_level(log::LevelFilter::Warn),
        1 => log::set_max_level(log::LevelFilter::Info),
        2 => log::set_max_level(log::LevelFilter::Debug),
        _ => log::set_max_level(log::LevelFilter::Trace),
    }
    debug!("Debuglevel: {}", log::max_level());

    let filter = matches
        .values_of_lossy("interfaces")
        .map(HashSet::from_iter);

    if let Some(filter) = &filter {
        debug!("Interfaces: {filter:?}");
    } else {
        debug!("Interfaces: ALL");
    }

    let interfaces = platform_interfaces().unwrap();

    match matches.subcommand() {
        ("find", Some(args)) => {
            let addr = EtherAddr::from_str(&args.value_of_lossy("device").unwrap()).unwrap();
            if let Some((interface, _)) = find_device(interfaces, filter, addr).unwrap() {
                println!("{addr:?}: Found on {interface}");
            } else {
                println!("{addr:?}: Not found");
            }
        }
        ("set-name", Some(args)) => {
            let addr = EtherAddr::from_str(&args.value_of_lossy("device").unwrap()).unwrap();
            let name = args.value_of_lossy("name").unwrap();
            if let Some((interface, oui)) = find_device(interfaces, filter, addr).unwrap() {
                set_name(interface, addr, oui, &name).unwrap();
            } else {
                println!("{addr:?}: Not found");
            }
        }
        ("dump", _) => {
            dump(interfaces);
        }
        ("scan", _) | ("", _) => {
            scan(interfaces, filter);
        }
        _ => panic!(),
    }
}
