#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MMV(pub u8);
impl MMV {
    pub const HOMEPLUG_AV_1_0: MMV = MMV(0x00);
    pub const HOMEPLUG_AV_1_1: MMV = MMV(0x01);
    pub const HOMEPLUG_AV_2_0: MMV = MMV(0x02);
}
impl core::fmt::Debug for MMV {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Self::HOMEPLUG_AV_1_0 => write!(f, "HPAV1.0"),
            Self::HOMEPLUG_AV_1_1 => write!(f, "HPAV1.1"),
            Self::HOMEPLUG_AV_2_0 => write!(f, "HPAV2.0"),
            _ => write!(f, "HPAV???({})", self.0),
        }
    }
}
