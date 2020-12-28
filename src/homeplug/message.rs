use super::*;

mod offset {
    pub const MMV: usize = 0;
    pub const MMTYPE_L: usize = 1;
    pub const MMTYPE_H: usize = 2;
    pub const FMI: usize = 3;
    pub const FMSN: usize = 4;
}

pub trait Message {
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
            MMV::HOMEPLUG_AV_1_1 | MMV::HOMEPLUG_AV_2_0 => &self.message_data()[5..],
            _ => &self.message_data()[0..0],
        }
    }
    // TODO: Fragmentation support for HPAV1.1/2.0 messages
}

pub struct Header<'a>(pub &'a [u8]);
impl Message for Header<'_> {
    fn message_data(&self) -> &[u8] {
        self.0
    }
}

pub fn hpav_set_header(data: &mut [u8], mmv: MMV, mmtype: MMType) -> &mut [u8] {
    data[offset::MMV] = mmv.0;
    data[offset::MMTYPE_L] = mmtype.to_le_bytes()[0];
    data[offset::MMTYPE_H] = mmtype.to_le_bytes()[1];
    match mmv {
        MMV::HOMEPLUG_AV_1_0 => &mut data[3..],
        MMV::HOMEPLUG_AV_1_1 | MMV::HOMEPLUG_AV_2_0 => {
            data[offset::FMI] = 0;
            data[offset::FMSN] = 0;
            &mut data[5..]
        }
        _ => &mut data[0..0],
    }
}
