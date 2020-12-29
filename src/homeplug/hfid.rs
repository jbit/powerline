use super::*;
use core::str;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct HFIDRequest(pub u8);
impl HFIDRequest {
    pub const GET_MFG: HFIDRequest = HFIDRequest(0x00);
    pub const GET_USR: HFIDRequest = HFIDRequest(0x01);
    pub const GET_NET: HFIDRequest = HFIDRequest(0x02);
    pub const SET_USR: HFIDRequest = HFIDRequest(0x03);
    pub const SET_NET: HFIDRequest = HFIDRequest(0x04);
    pub const FAILURE: HFIDRequest = HFIDRequest(0xff);
}
impl core::fmt::Debug for HFIDRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            HFIDRequest::GET_MFG => write!(f, "GET_MFG"),
            HFIDRequest::GET_USR => write!(f, "GET_USR"),
            HFIDRequest::GET_NET => write!(f, "GET_NET"),
            HFIDRequest::SET_USR => write!(f, "SET_USR"),
            HFIDRequest::SET_NET => write!(f, "SET_NET"),
            HFIDRequest::FAILURE => write!(f, "FAILURE"),
            _ => write!(f, "HFID{:02x}", self.0),
        }
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct HFID<'a>(pub &'a [u8]);
impl HFID<'_> {
    pub fn request(&self) -> HFIDRequest {
        HFIDRequest(self.payload()[0])
    }
    pub fn hfid_bytes(&self) -> &[u8] {
        &self.payload()[1..=65]
    }
    pub fn hfid(&self) -> &str {
        str::from_utf8(self.hfid_bytes()).unwrap_or_default()
    }
}
impl Message for HFID<'_> {
    const MMV: MMV = MMV::HOMEPLUG_AV_1_1;
    const MMTYPE: MMType = MMType::CM_HFID;
    fn message_data(&self) -> &[u8] {
        &self.0
    }
}
impl core::fmt::Debug for HFID<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}: {}", self.request(), self.hfid())
    }
}
impl<'a> From<&'a [u8]> for HFID<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self(data)
    }
}
