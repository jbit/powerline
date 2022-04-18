use super::*;
use crate::OUI;

mod offset {
    pub const MMV: usize = 0;
    pub const MMTYPE_L: usize = 1;
    pub const MMTYPE_H: usize = 2;
    pub const FMI: usize = 3;
    pub const FMSN: usize = 4;
    pub const OUI0: usize = 5;
    pub const OUI1: usize = 6;
    pub const OUI2: usize = 7;
}

pub trait MessageTX<'a> {
    const MMV: MMV;
    const MMTYPE: MMType;
    const OUI: OUI = OUI([0x00, 0x00, 0x00]);
    type Response: MessageReader;

    /// Set the payload bytes for this message transmission
    fn set_payload(&self, bytes: &mut [u8]) -> usize {
        let _ = bytes;
        0
    }

    /// Convert this transmissiable message to bytes
    fn encode<'b>(&self, bytes: &'b mut [u8]) -> &'b [u8] {
        let header_size = set_header(bytes, Self::MMV, Self::MMTYPE, Self::OUI);
        let payload_size = self.set_payload(&mut bytes[header_size..]);
        &bytes[..header_size + payload_size]
    }
}

pub trait MessageReader: core::fmt::Debug {
    /// Get all bytes of the message
    fn bytes(&self) -> &[u8];

    /// Management Message Version
    fn mmv(&self) -> MMV {
        MMV(self.bytes()[offset::MMV])
    }
    /// Management Message Type
    fn mmtype(&self) -> MMType {
        MMType::from_le_bytes([
            self.bytes()[offset::MMTYPE_L],
            self.bytes()[offset::MMTYPE_H],
        ])
    }
    /// Fragmentation Management Information (v1.1 and v2.0)
    fn fmi(&self) -> u8 {
        if self.mmv() == MMV::HOMEPLUG_AV_1_1 || self.mmv() == MMV::HOMEPLUG_AV_2_0 {
            self.bytes()[offset::FMI]
        } else {
            0
        }
    }
    /// Fragmentation Message Sequence Number (v1.1 and v2.0)
    fn fmsn(&self) -> u8 {
        if self.mmv() == MMV::HOMEPLUG_AV_1_1 || self.mmv() == MMV::HOMEPLUG_AV_2_0 {
            self.bytes()[offset::FMSN]
        } else {
            0
        }
    }
    /// Organizationally Unique Identifier (for vendor specific messages)
    fn oui(&self) -> OUI {
        if self.mmtype().is_vendor() {
            OUI([
                self.bytes()[offset::OUI0],
                self.bytes()[offset::OUI1],
                self.bytes()[offset::OUI2],
            ])
        } else {
            Default::default()
        }
    }
    /// Get payload bytes of the message
    fn payload(&self) -> &[u8] {
        let offset = header_size(self.mmv(), self.mmtype());
        &self.bytes()[offset..]
    }
}

pub struct UnknownMessage<'a>(pub &'a [u8]);
impl MessageReader for UnknownMessage<'_> {
    fn bytes(&self) -> &[u8] {
        self.0
    }
}
impl core::fmt::Debug for UnknownMessage<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "MMV:{:?} MMType:{:?}", self.mmv(), self.mmtype())?;
        Ok(())
    }
}

fn header_size(mmv: MMV, mmtype: MMType) -> usize {
    match mmv {
        MMV::HOMEPLUG_AV_1_1 | MMV::HOMEPLUG_AV_2_0 if mmtype.is_vendor() => 8,
        MMV::HOMEPLUG_AV_1_1 | MMV::HOMEPLUG_AV_2_0 => 5,
        MMV::HOMEPLUG_AV_1_0 => 3,
        _ => usize::MAX,
    }
}

fn set_header(bytes: &mut [u8], mmv: MMV, mmtype: MMType, oui: OUI) -> usize {
    let mut header = [0; 8];
    header[offset::MMV] = mmv.0;
    header[offset::MMTYPE_L] = mmtype.to_le_bytes()[0];
    header[offset::MMTYPE_H] = mmtype.to_le_bytes()[1];
    header[offset::FMI] = 0;
    header[offset::FMSN] = 0;
    header[offset::OUI0] = oui[0];
    header[offset::OUI1] = oui[1];
    header[offset::OUI2] = oui[2];

    let size = header_size(mmv, mmtype);
    bytes[..size].copy_from_slice(&header[..size]);
    size
}
