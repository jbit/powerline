#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct EtherType(pub u16);
impl EtherType {
    pub const LLDP: EtherType = EtherType(0x88cc);
    pub const HOMEPLUG: EtherType = EtherType(0x887b);
    pub const HOMEPLUG_AV: EtherType = EtherType(0x88e1);
    pub const MEDIAXTREAM: EtherType = EtherType(0x8912);
    pub const IEEE1905: EtherType = EtherType(0x893a);

    pub fn name(&self) -> Option<&'static str> {
        Some(match self {
            &EtherType::HOMEPLUG => "HomePlug",
            &EtherType::HOMEPLUG_AV => "HomePlug AV",
            &EtherType::MEDIAXTREAM => "Mediaxtream",
            &EtherType::IEEE1905 => "IEEE 1905",
            _ => return None,
        })
    }
    pub fn as_be_u16(&self) -> u16 {
        self.0.to_be()
    }
}
impl std::fmt::Debug for EtherType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Some(name) = self.name() {
            write!(f, "{}", name)
        } else {
            write!(f, "Unknown(0x{:04x})", self.0)
        }
    }
}
