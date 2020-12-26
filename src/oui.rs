use std::ops::Deref;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct OUI(pub [u8; 3]);
impl OUI {
    pub const QUALCOMM: OUI = OUI([0x00, 0xb0, 0x52]); // Qualcomm and Atheros devices
    pub const BROADCOM: OUI = OUI([0x00, 0x1f, 0x84]); // Broadcom and Gigle devices
    pub const ST: OUI = OUI([0x00, 0x80, 0xe1]); // ST and IoTecha devices

    pub fn name(&self) -> Option<&'static str> {
        Some(match self {
            &OUI::QUALCOMM => "Qualcomm",
            &OUI::BROADCOM => "Broadcom",
            &OUI::ST => "ST",
            _ => return None,
        })
    }
}
impl Deref for OUI {
    type Target = [u8; 3];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for OUI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}", name)
        } else {
            write!(
                f,
                "Unknown({:02x}:{:02x}:{:02x})",
                self[0], self[1], self[2]
            )
        }
    }
}
