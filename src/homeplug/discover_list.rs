use super::*;
use crate::*;
use core::ops::Deref;

pub struct Station<'a>(&'a [u8]);
impl core::fmt::Debug for Station<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let data = self.0;
        let sta_mac = EtherAddr::from_slice(&data[0..=5]);
        let tei = data[6];
        let same_network = data[7];
        let snid = data[8];
        let _flags = data[9];
        let sta_level = match data[10] {
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
        let ble = data[11];
        write!(
            f,
            "STA[{:?}] tei={} same_network:{} snid:{} level:{} ble:{}",
            sta_mac, tei, same_network, snid, sta_level, ble
        )?;

        Ok(())
    }
}

pub struct Network<'a>(&'a [u8]);
impl core::fmt::Debug for Network<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let data = self.0;
        let nid = &data[0..=6];
        let snid = data[7];
        let hybrid = data[8];
        let slots = data[9];
        let coordinating = data[10];
        let offset = data[11];
        write!(
            f,
            "NET[{:02x?}/{}] hybrid={} slots={} coordinating={} offset={}",
            nid, snid, hybrid, slots, coordinating, offset
        )?;

        Ok(())
    }
}

pub struct DiscoverList<T: Deref<Target = [u8]>>(pub EtherAddr, pub T);
impl<T: Deref<Target = [u8]>> DiscoverList<T> {
    pub fn stations(&self) -> impl ExactSizeIterator + Iterator<Item = Station> {
        let data = self.payload();
        let station_count = data[0] as usize;
        data[1..]
            .chunks_exact(12)
            .take(station_count)
            .map(|data| Station(data))
    }
    pub fn networks(&self) -> impl ExactSizeIterator + Iterator<Item = Network> {
        let data = self.payload();
        let station_count = data[0] as usize;
        let network_offset = 1 + (station_count * 20);
        let (network_count, networks) = if network_offset < data.len() {
            (data[network_offset] as usize, &data[network_offset + 1..])
        } else {
            (0, &data[0..0])
        };
        networks
            .chunks_exact(13)
            .take(network_count)
            .map(|data| Network(data))
    }
}
impl<T: Deref<Target = [u8]>> Message for DiscoverList<T> {
    fn message_data(&self) -> &[u8] {
        &self.1
    }
}
impl<T: Deref<Target = [u8]>> core::fmt::Debug for DiscoverList<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        writeln!(f, "[{:?}] {:?}:{:?}", self.0, self.mmv(), self.mmtype())?;
        for i in self.stations() {
            writeln!(f, "  {:?}", i)?;
        }
        for i in self.networks() {
            writeln!(f, "  {:?}", i)?;
        }
        Ok(())
    }
}
