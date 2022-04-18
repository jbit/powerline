use super::*;
use core::str;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct HFIDReqType(pub u8);
impl HFIDReqType {
    pub const GET_MFG: Self = Self(0x00);
    pub const GET_USR: Self = Self(0x01);
    pub const GET_NET: Self = Self(0x02);
    pub const SET_USR: Self = Self(0x03);
    pub const SET_NET: Self = Self(0x04);
    pub const FAILURE: Self = Self(0xff);
}
impl core::fmt::Debug for HFIDReqType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Self::GET_MFG => write!(f, "GET_MFG"),
            Self::GET_USR => write!(f, "GET_USR"),
            Self::GET_NET => write!(f, "GET_NET"),
            Self::SET_USR => write!(f, "SET_USR"),
            Self::SET_NET => write!(f, "SET_NET"),
            Self::FAILURE => write!(f, "FAILURE"),
            _ => write!(f, "HFIDReqType{:02x}", self.0),
        }
    }
}

pub enum HFIDRequest {
    GetMfg,
    GetUsr,
    GetNet { nid: [u8; 6] },
    SetUsr { hfid: [u8; 64] },
    SetNet { nid: [u8; 6], hfid: [u8; 64] },
}
impl<'a> MessageTX<'a> for HFIDRequest {
    const MMV: MMV = MMV::HOMEPLUG_AV_1_1;
    const MMTYPE: MMType = MMType::CM_HFID;
    type Response = HFID<'a>;

    fn set_payload(&self, bytes: &mut [u8]) -> usize {
        match self {
            HFIDRequest::GetMfg => {
                bytes[0] = HFIDReqType::GET_MFG.0;
                1
            }
            HFIDRequest::GetUsr => {
                bytes[0] = HFIDReqType::GET_USR.0;
                1
            }
            HFIDRequest::GetNet { nid } => {
                bytes[0] = HFIDReqType::GET_NET.0;
                bytes[1..7].copy_from_slice(nid);
                7
            }
            HFIDRequest::SetUsr { hfid } => {
                bytes[0] = HFIDReqType::SET_USR.0;
                bytes[1..65].copy_from_slice(hfid);
                65
            }
            HFIDRequest::SetNet { nid, hfid } => {
                bytes[0] = HFIDReqType::SET_NET.0;
                bytes[1..7].copy_from_slice(nid);
                bytes[7..71].copy_from_slice(hfid);
                71
            }
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct HFID<'a>(pub &'a [u8]);
impl HFID<'_> {
    pub fn req_type(&self) -> HFIDReqType {
        HFIDReqType(self.payload()[0])
    }
    pub fn hfid_bytes(&self) -> &[u8] {
        &self.payload()[1..=65]
    }
    pub fn hfid(&self) -> &str {
        str::from_utf8(self.hfid_bytes())
            .unwrap_or_default()
            .trim_end_matches('\0')
    }
}
impl MessageReader for HFID<'_> {
    fn bytes(&self) -> &[u8] {
        self.0
    }
}
impl core::fmt::Debug for HFID<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.req_type(), self.hfid())
    }
}
impl<'a> From<&'a [u8]> for HFID<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self(data)
    }
}
