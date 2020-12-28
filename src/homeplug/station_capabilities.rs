use super::*;
use crate::*;
use core::convert::TryInto;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct Version(pub u8);
impl Version {
    pub const HOMEPLUG_AV_1_1: Version = Version(0x00);
    pub const HOMEPLUG_AV_2_0: Version = Version(0x01);
}
impl core::fmt::Debug for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Version::HOMEPLUG_AV_1_1 => write!(f, "HPAV1.1"),
            Version::HOMEPLUG_AV_2_0 => write!(f, "HPAV2.0"),
            _ => write!(f, "HPAV???(0x{:02x})", self.0),
        }
    }
}

pub struct StationCapabilities<'a>(pub &'a [u8]);
impl StationCapabilities<'_> {
    pub fn version(&self) -> Version {
        Version(self.payload()[0])
    }
    pub fn addr(&self) -> EtherAddr {
        EtherAddr::from_slice(&self.payload()[1..=6])
    }
    pub fn oui(&self) -> OUI {
        OUI(self.payload()[7..=9].try_into().unwrap())
    }
}
impl Message for StationCapabilities<'_> {
    const MMV: MMV = MMV::HOMEPLUG_AV_1_1;
    const MMTYPE: MMType = MMType::CM_STA_CAP;
    fn message_data(&self) -> &[u8] {
        &self.0
    }
}
impl core::fmt::Debug for StationCapabilities<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let data = self.payload();
        let version = self.version();
        let _addr = self.addr();
        let oui = self.oui();

        write!(f, "Capabilities({:?} OUI={:?}", version, oui)?;
        if data[10] != 0 {
            write!(f, " AutoConnect")?;
        }
        if data[11] != 0 {
            write!(f, " Smoothing")?;
        }
        write!(f, " CCoLevel{}", data[12])?;
        if data[13] != 0 {
            write!(f, " Proxy")?;
        }
        if data[14] != 0 {
            write!(f, " Cap14={}", data[14])?;
        }
        if data[15] != 0 {
            write!(f, " BackupCCo")?;
        }
        if data[16] != 0 {
            write!(f, " SoftHandOver")?;
        }
        if data[17] != 0 {
            write!(f, " TwoSymFC")?;
        }
        let maxflav = u16::from_le_bytes([data[18], data[19]]) as f32 * 1.28;
        write!(f, " MaxFrameLength={:.2}uS", maxflav)?;
        if data[20] != 0 {
            write!(f, " HP1.1")?;
        }
        if data[21] != 0 {
            write!(f, " HP1.0.1")?;
        }
        match data[22] {
            0 => write!(f, " NorthAmerica")?,
            x => write!(f, " Region{}", x)?,
        }
        match data[23] {
            0 => {}
            1 => write!(f, " Burst=SACK")?,
            2 => write!(f, " Burst=SACK+SOF")?,
            x => write!(f, " Burst=0x{:02x}", x)?,
        }
        if data[24] != 0 || data[25] != 0 {
            write!(f, " Ver={}.{}", data[24], data[25])?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
impl<'a> From<&'a [u8]> for StationCapabilities<'a> {
    fn from(data: &'a [u8]) -> Self {
        Self(data)
    }
}
