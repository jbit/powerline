use powerline::{linux::*, *};

const MMV: usize = 0;
const MMTYPE_L: usize = 1;
const MMTYPE_H: usize = 2;
const FMI: usize = 3;
const FMSN: usize = 4;

const MMV_HOMEPLUG_AV_1_0: u8 = 0x00;
const MMV_HOMEPLUG_AV_1_1: u8 = 0x01;
const MMV_HOMEPLUG_AV_2_0: u8 = 0x02;

fn discover_stations<T: EtherSocket>(socket: &mut T) -> Result<(), T::Error> {
    let mmtype = MMType::CC_DISCOVER_LIST;
    let mut message = [0u8; 60];
    message[MMV] = MMV_HOMEPLUG_AV_1_1;
    message[MMTYPE_L] = mmtype.to_le_bytes()[0];
    message[MMTYPE_H] = mmtype.to_le_bytes()[1];
    message[FMI] = 0;
    message[FMSN] = 0;
    socket.sendto(EtherAddr::BROADCAST, &message)?;

    let mut buffer = [0; 1500];
    loop {
        let (addr, msg) = socket.recvfrom(&mut buffer)?;
        let mmv = msg[MMV];
        let mmtype = MMType::from_le_bytes([msg[MMTYPE_L], msg[MMTYPE_H]]);
        println!("[{:?}] MMV:{} MMType:{:?} ", addr, mmv, mmtype);

        let mmentry = match mmv {
            MMV_HOMEPLUG_AV_1_0 => 3,
            MMV_HOMEPLUG_AV_1_1 => 5,
            MMV_HOMEPLUG_AV_2_0 => 5,
            _ => continue,
        };

        let data = &msg[mmentry..];

        use MMTypeCode::*;
        match mmtype.split() {
            (MMType::CM_MME_ERROR, IND) => {
                let error = match data[23] {
                    0x00 => "Not Supported".to_string(),
                    0x01 => "Invalid fields".to_string(),
                    0x02 => "Unsupported feature".to_string(),
                    code => format!("Unknown Reason({:02x})", code),
                };
                let error_mmv = data[1];
                let error_mmtype = MMType::from_le_bytes([data[2], data[3]]);
                let offset = u16::from_le_bytes([data[4], data[5]]);
                println!(
                    "MME Error! {} MMV:{} MMType:{:?} Offset:{}",
                    error, error_mmv, error_mmtype, offset
                );
            }
            (MMType::CC_DISCOVER_LIST, CNF) => {
                let (&station_count, data) = data.split_first().unwrap();
                let (station_data, data) = data.split_at(12 * station_count as usize);
                for station in station_data.chunks_exact(12) {
                    let sta_mac = EtherAddr::from_slice(&station[0..=5]);
                    let tei = station[6];
                    let same_network = station[7];
                    let snid = station[8];
                    let _flags = station[9];
                    let sta_level = match station[10] {
                        0x00 => "Unknown",
                        0x01 => ">-10dB",
                        0x02 => ">-15dB",
                        0x03 => ">-20dB",
                        0x04 => ">-25dB",
                        0x05 => ">-30dB",
                        0x06 => ">-35dB",
                        0x07 => ">-40dB",
                        0x08 => ">-45dB",
                        0x09 => ">-50dB",
                        0x0a => ">-55dB",
                        0x0b => ">-60dB",
                        0x0c => ">-65dB",
                        0x0d => ">-70dB",
                        0x0e => ">-75dB",
                        0x0f => "<-75dB",
                        _ => "????",
                    };
                    let ble = station[11];
                    println!(
                        "  STA[{:?}] tei={} same_network:{} snid:{} level:{} ble:{}",
                        sta_mac, tei, same_network, snid, sta_level, ble
                    );
                }

                // TODO:
                let (&network_count, data) = data.split_first().unwrap();
                let (network_data, _) = data.split_at(13 * network_count as usize);
                for network in network_data.chunks_exact(13) {
                    let nid = &network[0..=6];
                    let snid = network[7];
                    let hybrid = network[8];
                    let slots = network[9];
                    let coordinating = network[10];
                    let offset = network[11];
                    println!(
                        "  NET[{:02x?}/{}] hybrid={} slots={} coordinating={} offset={}",
                        nid, snid, hybrid, slots, coordinating, offset
                    );
                }
            }
            _ => {
                println!("{:?} unknown packet: {:02x?}", mmtype, data);
            }
        }
    }
}

fn main() {
    for interface in LinuxInterface::interfaces().unwrap() {
        if !interface.is_up() || interface.is_loopback() {
            continue;
        }
        println!("-- Interface: {:?}", interface);
        let mut socket = interface.open(EtherType::HOMEPLUG_AV).unwrap();
        discover_stations(&mut socket).unwrap();
    }
}
