use super::OUI;
use core::convert::TryInto;
use core::str::FromStr;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct EtherAddr(pub [u8; 6]);
impl EtherAddr {
    pub const NULL: EtherAddr = EtherAddr([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    pub const BROADCAST: EtherAddr = EtherAddr([0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    pub const QUALCOMM_LOCALCAST: EtherAddr = EtherAddr([0x00, 0xb0, 0x52, 0x00, 0x00, 0x01]);
    pub const IEEE1905_MULTICAST: EtherAddr = EtherAddr([0x01, 0x80, 0xc2, 0x00, 0x00, 0x13]);

    pub fn from_slice(slice: &[u8]) -> EtherAddr {
        EtherAddr(slice.try_into().unwrap())
    }
    pub fn oui(&self) -> OUI {
        OUI([self[0], self[1], self[2]])
    }
    pub fn padded(&self) -> [u8; 8] {
        [self[0], self[1], self[2], self[3], self[4], self[5], 0, 0]
    }
    pub fn is_unicast(&self) -> bool {
        !self.is_multicast()
    }
    pub fn is_multicast(&self) -> bool {
        self[0] & 1 == 1
    }
    pub fn is_broadcast(&self) -> bool {
        *self == Self::BROADCAST
    }
}
impl core::ops::Deref for EtherAddr {
    type Target = [u8; 6];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl core::fmt::Debug for EtherAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self[0], self[1], self[2], self[3], self[4], self[5]
        )
    }
}
impl FromStr for EtherAddr {
    type Err = ();
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim();
        let mut addr = [0u8; 6];
        let mut i = 0;
        for part in input.split(|c| c == ':' || c == '-') {
            if i >= addr.len() {
                return Err(());
            }
            let part = part.trim();
            if part.len() != 2 {
                return Err(());
            }
            let value = u8::from_str_radix(part, 16).map_err(|_| ())?;
            addr[i] = value;
            i += 1;
        }

        if i != 6 {
            return Err(());
        }

        Ok(EtherAddr(addr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn properties() {
        assert_eq!(EtherAddr::BROADCAST.is_unicast(), false);
        assert_eq!(EtherAddr::BROADCAST.is_multicast(), true);
        assert_eq!(EtherAddr::BROADCAST.is_broadcast(), true);
        assert_eq!(EtherAddr::BROADCAST.oui(), OUI([0xff, 0xff, 0xff]));

        assert_eq!(EtherAddr::QUALCOMM_LOCALCAST.is_unicast(), true);
        assert_eq!(EtherAddr::QUALCOMM_LOCALCAST.is_multicast(), false);
        assert_eq!(EtherAddr::QUALCOMM_LOCALCAST.is_broadcast(), false);
        assert_eq!(EtherAddr::QUALCOMM_LOCALCAST.oui(), OUI::QUALCOMM);

        assert_eq!(EtherAddr::IEEE1905_MULTICAST.is_unicast(), false);
        assert_eq!(EtherAddr::IEEE1905_MULTICAST.is_multicast(), true);
        assert_eq!(EtherAddr::IEEE1905_MULTICAST.is_broadcast(), false);
        assert_eq!(EtherAddr::IEEE1905_MULTICAST.oui(), OUI([0x01, 0x80, 0xc2]));
    }

    #[test]
    fn from_str() {
        // Test valid inputs
        for test in &[
            "01:23:45:67:89:ab",
            "01-23-45-67-89-ab",
            "01-23-45-67-89-AB",
            "01-23-45-67-89-Ab",
            "\t01-23-45-67-89-ab  ",
            "  01 : 23 : 45 : 67 : 89 : ab  ",
        ] {
            let model = Ok(EtherAddr([0x01, 0x23, 0x45, 0x67, 0x89, 0xab]));
            assert_eq!(model, EtherAddr::from_str(test), "\"{}\"", test);
        }

        // Test invalid inputs

        // Various invalid addresses
        for test in &[
            "",
            "0",
            "01",
            "01-",
            "01-2",
            "01-23",
            "01-23-",
            "01-23-4",
            "01-23-45",
            "01-23-45-",
            "01-23-45-6",
            "01-23-45-67",
            "01-23-45-67-",
            "01-23-45-67-8",
            "01-23-45-67-89",
            "01-23-45-67-89-",
            "01-23-45-67-89-a",
            "xx-xx-xx-xx-xx-xx",
            "01-23-45-67-89-xx",
            "01-23-45-67-89-ab-",
            "01-23-45-67-89-ab-cd",
            "-01-23-45-67-89-ab",
            "01--23-45-67-89-ab",
            "0 1-23-45-67-89-ab",
            "1-2-3-4-5-6",
            "-----",
            ":::::",
        ] {
            assert_eq!(Err(()), EtherAddr::from_str(test), "\"{}\"", test);
        }
    }
}
