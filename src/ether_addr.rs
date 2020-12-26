use super::OUI;
use std::convert::TryInto;
use std::ops::Deref;

#[repr(transparent)]
#[derive(Default, PartialEq, Copy, Clone)]
pub struct EtherAddr(pub [u8; 6]);
impl EtherAddr {
    pub const NULL: EtherAddr = EtherAddr([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    pub const BROADCAST: EtherAddr = EtherAddr([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    pub const QUALCOMM_LOCALCAST: EtherAddr = EtherAddr([0x00, 0xb0, 0x52, 0x00, 0x00, 0x01]);

    pub fn from_slice(slice: &[u8]) -> EtherAddr {
        EtherAddr(slice.try_into().unwrap())
    }
    pub fn oui(&self) -> OUI {
        OUI([self[0], self[1], self[2]])
    }
    pub fn padded(&self) -> [u8; 8] {
        [self[0], self[1], self[2], self[3], self[4], self[5], 0, 0]
    }
}
impl Deref for EtherAddr {
    type Target = [u8; 6];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for EtherAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self[0], self[1], self[2], self[3], self[4], self[5]
        )
    }
}
