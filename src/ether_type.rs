use core::convert::TryInto;
use core::mem::size_of;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct EtherType(pub u16);
impl EtherType {
    pub const SIZE: usize = size_of::<Self>();
    pub const LLDP: EtherType = EtherType(0x88cc);
    pub const HOMEPLUG: EtherType = EtherType(0x887b);
    pub const HOMEPLUG_AV: EtherType = EtherType(0x88e1);
    pub const MEDIAXTREAM: EtherType = EtherType(0x8912);
    pub const IEEE1905: EtherType = EtherType(0x893a);

    pub fn name(&self) -> Option<&'static str> {
        Some(match *self {
            EtherType::HOMEPLUG => "HomePlug",
            EtherType::HOMEPLUG_AV => "HomePlug AV",
            EtherType::MEDIAXTREAM => "Mediaxtream",
            EtherType::IEEE1905 => "IEEE 1905",
            _ => return None,
        })
    }
    pub fn from_slice(buf: &[u8]) -> EtherType {
        EtherType(u16::from_be_bytes(buf.try_into().unwrap()))
    }
    pub fn as_be_u16(&self) -> u16 {
        self.0.to_be()
    }
    pub fn as_bytes(&self) -> [u8; EtherType::SIZE] {
        self.0.to_be_bytes()
    }
}
impl core::fmt::Debug for EtherType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}", name)
        } else {
            write!(f, "Unknown(0x{:04x})", self.0)
        }
    }
}
