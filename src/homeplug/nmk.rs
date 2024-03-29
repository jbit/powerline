#![allow(non_upper_case_globals)]

pub const NMK_HomePlugAV: [u8; 16] = [
    0x50, 0xd3, 0xe4, 0x93, 0x3f, 0x85, 0x5b, 0x70, 0x40, 0x78, 0x4d, 0xf8, 0x15, 0xaa, 0x8d, 0xb7,
];
pub const NMK_HomePlugAV0123: [u8; 16] = [
    0xb5, 0x93, 0x19, 0xd7, 0xe8, 0x15, 0x7b, 0xa0, 0x01, 0xb0, 0x18, 0x66, 0x9c, 0xce, 0xe3, 0x0d,
];

#[cfg(feature = "sha2")]
pub fn generate_nmk(s: &str) -> [u8; 16] {
    use core::convert::TryInto;
    use sha2::{Digest, Sha256};
    let magic = [0x08, 0x85, 0x6d, 0xaf, 0x7c, 0xf5, 0x81, 0x86];
    let mut hasher = Sha256::new();
    hasher.update(s);
    hasher.update(magic);
    let mut hash = hasher.finalize();
    for _ in 0..999 {
        let mut hasher = Sha256::new();
        hasher.update(hash);
        hash = hasher.finalize();
    }
    hash[..16].try_into().unwrap()
}

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct SecurityLevel(pub u8);
impl SecurityLevel {
    pub const SIMPLE: Self = Self(0x00);
    pub const SECURE: Self = Self(0x01);
}
impl core::fmt::Debug for SecurityLevel {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Self::SIMPLE => write!(f, "SIMPLE"),
            Self::SECURE => write!(f, "SECURE"),
            _ => write!(f, "SecurityLevel{:02x}", self.0),
        }
    }
}

#[cfg(feature = "sha2")]
pub fn generate_nid(nmk: [u8; 16], security: SecurityLevel) -> [u8; 7] {
    use core::convert::TryInto;
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(nmk);
    let mut hash = hasher.finalize();
    for _ in 0..4 {
        let mut hasher = Sha256::new();
        hasher.update(hash);
        hash = hasher.finalize();
    }
    hash[6] = (hash[6] >> 4) | (security.0 << 4);
    hash[..7].try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_keys() {
        assert_eq!(generate_nmk("HomePlugAV"), NMK_HomePlugAV);
        assert_eq!(generate_nmk("HomePlugAV0123"), NMK_HomePlugAV0123);
    }

    #[test]
    #[rustfmt::skip]
    fn other_keys() {
        assert_eq!(
            generate_nmk("The quick brown fox jumped over the lazy dog."),
            [0x56, 0xf3, 0xc7, 0xf5, 0x39, 0xd4, 0xf8, 0xf5, 0xee, 0xc0, 0x0e, 0x63, 0xf1, 0x1a, 0x8d, 0xec],
        );
        assert_eq!(
            generate_nmk("-HomePlugAV"),
            [0x80, 0xb7, 0x4b, 0x14, 0xe9, 0x2a, 0x73, 0x9a, 0xd4, 0x1a, 0xcd, 0xc3, 0x77, 0x45, 0x1d, 0x1b],
        );
        assert_eq!(
            generate_nmk("-HomePlugAV123"),
            [0x1a, 0x46, 0xbd, 0xe6, 0xf7, 0x52, 0x09, 0x29, 0x2f, 0xdf, 0xc4, 0xcc, 0xe4, 0xd1, 0x9b, 0x4e],
        );
        assert_eq!(
            generate_nmk("01234567890123456789"),
            [0xf2, 0xb0, 0xc7, 0xf6, 0xc3, 0x55, 0x98, 0x1e, 0xbd, 0xd4, 0x84, 0xff, 0x49, 0x95, 0x74, 0x20],
        );
        assert_eq!(
            generate_nmk("abcdefghijklmnopqrstuvwxyz"),
            [0x54, 0xcb, 0x8a, 0xb1, 0x23, 0x58, 0x96, 0xe4, 0x5e, 0x6b, 0x64, 0x3c, 0x7b, 0xf1, 0x1a, 0xdb],
        );
        assert_eq!(
            generate_nmk(r#"~!@#$%^&*()_-`{}[]":;'\|<>./?"#),
            [0x16, 0x71, 0xd6, 0x1f, 0x30, 0x5e, 0x81, 0xba, 0xf0, 0x00, 0xd5, 0x8a, 0xf0, 0x98, 0x88, 0xd5],
        );
    }

    #[test]
    fn standard_nids() {
        assert_eq!(
            generate_nid(NMK_HomePlugAV, SecurityLevel::SIMPLE),
            [0xb0, 0xf2, 0xe6, 0x95, 0x66, 0x6b, 0x03]
        );
        assert_eq!(
            generate_nid(NMK_HomePlugAV0123, SecurityLevel::SECURE),
            [0x02, 0x6b, 0xcb, 0xa5, 0x35, 0x4e, 0x18]
        );
    }
}
