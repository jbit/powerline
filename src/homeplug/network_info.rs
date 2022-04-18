use super::*;
use crate::*;

pub struct NetworkInfoRequest;
impl<'a> MessageTX<'a> for NetworkInfoRequest {
    const MMV: MMV = MMV::HOMEPLUG_AV_1_1;
    const MMTYPE: MMType = MMType::CM_NW_INFO;
    type Response = NetworkInfo<'a>;
}

pub struct NetworkInfoEntry<'a>(&'a [u8]);
impl core::fmt::Debug for NetworkInfoEntry<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let data = self.0;
        let nid = &data[0..=6];
        let snid = data[7];
        let tei = data[8];
        let station_role = StationRole(data[9]);
        let cco_macaddr = EtherAddr::from_slice(&data[10..=15]);
        let access = data[16];
        let num_cord_nws = data[16];
        write!(
            f,
            "NET[{nid:02x?}/{snid}] tei={tei} role={station_role:?} CCo={cco_macaddr:?} access={access} neighbors={num_cord_nws}",
        )?;

        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct NetworkInfo<'a>(pub &'a [u8]);
impl NetworkInfo<'_> {
    pub fn networks(&self) -> impl ExactSizeIterator + Iterator<Item = NetworkInfoEntry> + '_ {
        let count = self.payload()[0] as usize;
        self.payload()[1..]
            .chunks_exact(18)
            .take(count)
            .map(NetworkInfoEntry)
    }
}
impl MessageReader for NetworkInfo<'_> {
    fn bytes(&self) -> &[u8] {
        self.0
    }
}
impl core::fmt::Debug for NetworkInfo<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for nw in self.networks() {
            writeln!(f, "{nw:?}")?;
        }
        Ok(())
    }
}
impl<'a> From<&'a [u8]> for NetworkInfo<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self(data)
    }
}

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct StationRole(pub u8);
impl StationRole {
    /// Station
    pub const STA: Self = Self(0x00);
    /// Proxy Coordinator
    pub const PCO: Self = Self(0x01);
    /// Central Coordinator
    pub const CCO: Self = Self(0x02);
}
impl core::fmt::Debug for StationRole {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Self::STA => write!(f, "STA"),
            Self::PCO => write!(f, "PCo"),
            Self::CCO => write!(f, "CCo"),
            _ => write!(f, "Result{:02x}", self.0),
        }
    }
}
