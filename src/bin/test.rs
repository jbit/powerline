use powerline::{homeplug::*, linux::*, *};
use std::time::Duration;

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
                println!("[{:?}] {:?}:{:?} - Unexpected message", addr, header.mmv(), header.mmtype());
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
        println!("-- Interface: {:?}", interface);
        let mut socket = interface.open(EtherType::HOMEPLUG_AV).unwrap();

        discover_list(&mut socket, |l| println!("{:?}", l)).unwrap();
    }
}
