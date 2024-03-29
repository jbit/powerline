use super::*;
use crate::*;

pub struct GetPropertyRequest {
    pub seq: u8,
    pub property: Property,
}
impl<'a> MessageTX<'a> for GetPropertyRequest {
    const MMV: MMV = MMV::HOMEPLUG_AV_2_0;
    const MMTYPE: MMType = MMType(0xa05c);
    const OUI: OUI = OUI::BROADCOM;
    type Response = GetProperty<'a>;

    fn set_payload(&self, bytes: &mut [u8]) -> usize {
        bytes[0] = self.seq;
        bytes[1] = self.property.0;
        2
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct GetProperty<'a>(pub &'a [u8]);
impl GetProperty<'_> {
    pub fn seq(&self) -> u8 {
        self.payload()[0]
    }
    pub fn count(&self) -> usize {
        self.payload()[1] as usize
    }
    pub fn record_size(&self) -> usize {
        u16::from_le_bytes([self.payload()[2], self.payload()[3]]) as usize
    }
    pub fn records(&self) -> impl ExactSizeIterator + Iterator<Item = &[u8]> {
        self.payload()[4..]
            .chunks_exact(self.record_size())
            .take(self.count())
    }
    pub fn first(&self) -> Option<&[u8]> {
        self.records().next()
    }
}
impl MessageReader for GetProperty<'_> {
    fn bytes(&self) -> &[u8] {
        self.0
    }
}
impl core::fmt::Debug for GetProperty<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "[")?;
        for record in self.records() {
            write!(f, " {:02x?} ", record)?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
impl<'a> From<&'a [u8]> for GetProperty<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self(data)
    }
}
