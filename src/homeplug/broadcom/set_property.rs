use super::*;
use crate::*;

mod offset {
    pub const SEQ: usize = 0;
    pub const PROPERTY: usize = 1;
    pub const UNKNOWN: usize = 2;
    pub const COUNT: usize = 3;
    pub const SIZE0: usize = 4;
    pub const SIZE1: usize = 5;
    pub const DATA: usize = 6;
}

pub struct SetPropertyRequest {
    pub seq: u8,
    pub property: broadcom::Property,
    pub data: [u8; 64],
}
impl<'a> MessageTX<'a> for SetPropertyRequest {
    const MMV: MMV = MMV::HOMEPLUG_AV_2_0;
    const MMTYPE: MMType = MMType(0xa058);
    const OUI: OUI = OUI::BROADCOM;
    type Response = SetProperty<'a>;

    fn set_payload(&self, bytes: &mut [u8]) -> usize {
        let record_size_bytes = (self.data.len() as u16).to_le_bytes();
        bytes[offset::SEQ] = self.seq;
        bytes[offset::PROPERTY] = self.property.0;
        bytes[offset::UNKNOWN] = 0x00;
        bytes[offset::COUNT] = 1;
        bytes[offset::SIZE0] = record_size_bytes[0];
        bytes[offset::SIZE1] = record_size_bytes[1];
        bytes[offset::DATA..]
            .iter_mut()
            .zip(self.data)
            .for_each(|(dest, src)| *dest = src);
        9
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct SetProperty<'a>(pub &'a [u8]);
impl SetProperty<'_> {}
impl MessageReader for SetProperty<'_> {
    fn bytes(&self) -> &[u8] {
        self.0
    }
}
impl core::fmt::Debug for SetProperty<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:02x?}", self.payload())
    }
}
impl<'a> From<&'a [u8]> for SetProperty<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self(data)
    }
}
