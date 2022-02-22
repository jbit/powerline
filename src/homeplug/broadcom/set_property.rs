use super::*;
use crate::*;

#[derive(Eq, PartialEq, Hash)]
pub struct SetProperty<'a>(pub &'a [u8]);
impl SetProperty<'_> {}
impl Message for SetProperty<'_> {
    const MMV: MMV = MMV::HOMEPLUG_AV_2_0;
    const MMTYPE: MMType = MMType(0xa058);
    const OUI: OUI = OUI::BROADCOM;
    fn message_data(&self) -> &[u8] {
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
