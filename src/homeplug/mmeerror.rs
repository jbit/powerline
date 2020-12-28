use super::*;

#[repr(transparent)]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub struct ErrorType(pub u8);
impl ErrorType {
    pub const NOT_SUPPORTED: ErrorType = ErrorType(0x00);
    pub const INVALID_FIELDS: ErrorType = ErrorType(0x01);
    pub const UNSUPPORTED_FEATURE: ErrorType = ErrorType(0x02);
}
impl core::fmt::Debug for ErrorType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            ErrorType::NOT_SUPPORTED => write!(f, "Not Supported"),
            ErrorType::INVALID_FIELDS => write!(f, "Invalid Fields"),
            ErrorType::UNSUPPORTED_FEATURE => write!(f, "Unsupported Feature"),
            _ => write!(f, "Unknown(0x{:02x})", self.0),
        }
    }
}

pub struct MMEError<'a>(pub &'a [u8]);
impl MMEError<'_> {
    pub fn error(&self) -> ErrorType {
        ErrorType(self.payload()[0])
    }
    pub fn error_mmv(&self) -> MMV {
        MMV(self.payload()[1])
    }
    pub fn error_mmtype(&self) -> MMType {
        MMType::from_le_bytes([self.payload()[2], self.payload()[3]])
    }
    pub fn error_offset(&self) -> usize {
        u16::from_le_bytes([self.payload()[4], self.payload()[5]]) as usize
    }
}
impl Message for MMEError<'_> {
    const MMV: MMV = MMV::HOMEPLUG_AV_1_1;
    const MMTYPE: MMType = MMType::CM_MME_ERROR;
    fn message_data(&self) -> &[u8] {
        &self.0
    }
}
impl core::fmt::Debug for MMEError<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "MMEError({:?} MMV:{:?} MMType:{:?} Offset:{})",
            self.error(),
            self.error_mmv(),
            self.error_mmtype(),
            self.error_offset()
        )
    }
}
