use super::*;
use crate::OUI;
use core::convert::TryInto;

mod offset {
    pub const MMV: usize = 0;
    pub const MMTYPE_L: usize = 1;
    pub const MMTYPE_H: usize = 2;
    pub const FMI: usize = 3;
    pub const FMSN: usize = 4;
}

pub trait Message: core::fmt::Debug {
    const MMV: MMV;
    const MMTYPE: MMType;
    const OUI: OUI = OUI([0x00, 0x00, 0x00]);
    fn message_data(&self) -> &[u8];
    fn mmv(&self) -> MMV {
        MMV(self.message_data()[offset::MMV])
    }
    fn mmtype(&self) -> MMType {
        MMType::from_le_bytes([
            self.message_data()[offset::MMTYPE_L],
            self.message_data()[offset::MMTYPE_H],
        ])
    }
    fn payload(&self) -> &[u8] {
        match self.mmv() {
            MMV::HOMEPLUG_AV_1_0 => &self.message_data()[3..],
            MMV::HOMEPLUG_AV_1_1 if self.mmtype().is_vendor() => &self.message_data()[8..],
            MMV::HOMEPLUG_AV_1_1 => &self.message_data()[5..],
            MMV::HOMEPLUG_AV_2_0 if self.mmtype().is_vendor() => &self.message_data()[8..],
            MMV::HOMEPLUG_AV_2_0 => &self.message_data()[5..],
            _ => &self.message_data()[0..0],
        }
    }
    fn mmoui(&self) -> OUI {
        if self.mmtype().is_vendor() {
            OUI(self.message_data()[5..=7].try_into().unwrap())
        } else {
            Default::default()
        }
    }
    // TODO: Fragmentation support for HPAV1.1/2.0 messages
}

pub struct UnknownMessage<'a>(pub &'a [u8]);
impl Message for UnknownMessage<'_> {
    const MMV: MMV = MMV(0xff);
    const MMTYPE: MMType = MMType(0xffff);
    fn message_data(&self) -> &[u8] {
        &self.0
    }
}
impl core::fmt::Debug for UnknownMessage<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MMV:{:?} MMType:{:?}", self.mmv(), self.mmtype())?;
        Ok(())
    }
}

pub fn hpav_set_header<M: Message>(data: &mut [u8]) -> &mut [u8] {
    data[offset::MMV] = M::MMV.0;
    data[offset::MMTYPE_L] = M::MMTYPE.to_le_bytes()[0];
    data[offset::MMTYPE_H] = M::MMTYPE.to_le_bytes()[1];
    match M::MMV {
        MMV::HOMEPLUG_AV_1_0 => &mut data[3..],
        MMV::HOMEPLUG_AV_1_1 | MMV::HOMEPLUG_AV_2_0 => {
            data[offset::FMI] = 0;
            data[offset::FMSN] = 0;
            if M::MMTYPE.is_vendor() {
                data[5] = M::OUI[0];
                data[6] = M::OUI[1];
                data[7] = M::OUI[2];
                &mut data[8..]
            } else {
                &mut data[5..]
            }
        }
        _ => &mut data[0..0],
    }
}
