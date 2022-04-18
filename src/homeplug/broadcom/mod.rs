use super::*;

mod get_property;
mod set_property;

pub use get_property::*;
pub use set_property::*;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct Property(pub u8);
impl Property {
    pub const NAME_A0: Property = Property(0x1b);
    pub const NAME_B0: Property = Property(0x1c);
    pub const HFID_USER: Property = Property(0x25);
    pub const NAME_B1: Property = Property(0x26);
}
