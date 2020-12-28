#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MMTypeCode {
    REQ = 0b00,
    CNF = 0b01,
    IND = 0b10,
    RSP = 0b11,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct MMType(pub u16);
impl MMType {
    pub const CC_CCO_APPOINT: MMType = MMType(0x0000);
    pub const CC_BACKUP_APPOINT: MMType = MMType(0x0004);
    pub const CC_LINK_INFO: MMType = MMType(0x0008);
    pub const CC_HANDOVER: MMType = MMType(0x000C);
    pub const CC_HANDOVER_INFO: MMType = MMType(0x0010);
    pub const CC_DISCOVER_LIST: MMType = MMType(0x0014);
    pub const CC_LINK_NEW: MMType = MMType(0x0018);
    pub const CC_LINK_MOD: MMType = MMType(0x001C);
    pub const CC_LINK_SQZ: MMType = MMType(0x0020);
    pub const CC_LINK_REL: MMType = MMType(0x0024);
    pub const CC_DETECT_REPORT: MMType = MMType(0x0028);
    pub const CC_WHO_RU: MMType = MMType(0x002C);
    pub const CC_ASSOC: MMType = MMType(0x0030);
    pub const CC_LEAVE: MMType = MMType(0x0034);
    pub const CC_SET_TEI_MAP: MMType = MMType(0x0038);
    pub const CC_RELAY: MMType = MMType(0x003C);
    pub const CC_BEACON_RELIABILITY: MMType = MMType(0x0040);
    pub const CC_ALLOC_MOVE: MMType = MMType(0x0044);
    pub const CC_ACCESS_NEW: MMType = MMType(0x0048);
    pub const CC_ACCESS_REL: MMType = MMType(0x004C);
    pub const CC_DCPPC: MMType = MMType(0x0050);
    pub const CC_HP1_DET: MMType = MMType(0x0054);
    pub const CC_BLE_UPDATE: MMType = MMType(0x0058);
    pub const CC_BCAST_REPEAT: MMType = MMType(0x005C);
    pub const CC_MH_LINK_NEW: MMType = MMType(0x0060);
    pub const CC_ISP_DETECTION_REPORT: MMType = MMType(0x0064);
    pub const CC_ISP_START_RESYNC: MMType = MMType(0x0068);
    pub const CC_ISP_FINISH_RESYNC: MMType = MMType(0x006C);
    pub const CC_ISP_RESYNC_DETECTED: MMType = MMType(0x0070);
    pub const CC_ISP_RESYNC_TRANSMIT: MMType = MMType(0x0074);
    pub const CC_POWERSAVE: MMType = MMType(0x0078);
    pub const CC_POWERSAVE_EXIT: MMType = MMType(0x007C);
    pub const CC_POWERSAVE_LIST: MMType = MMType(0x0080);
    pub const CC_STOP_POWERSAVE: MMType = MMType(0x0084);
    pub const CP_PROXY_APPOINT: MMType = MMType(0x2000);
    pub const PH_PROXY_APPOINT: MMType = MMType(0x2004);
    pub const CP_PROXY_WAKE: MMType = MMType(0x2008);
    pub const NN_INL: MMType = MMType(0x4000);
    pub const NN_NEW_NET: MMType = MMType(0x4004);
    pub const NN_ADD_ALLOC: MMType = MMType(0x4008);
    pub const NN_REL_ALLOC: MMType = MMType(0x400C);
    pub const NN_REL_NET: MMType = MMType(0x4010);
    pub const CM_UNASSOCIATED_STA: MMType = MMType(0x6000);
    pub const CM_ENCRYPTED_PAYLOAD: MMType = MMType(0x6004);
    pub const CM_SET_KEY: MMType = MMType(0x6008);
    pub const CM_GET_KEY: MMType = MMType(0x600C);
    pub const CM_SC_JOIN: MMType = MMType(0x6010);
    pub const CM_CHAN_EST: MMType = MMType(0x6014);
    pub const CM_TM_UPDATE: MMType = MMType(0x6018);
    pub const CM_AMP_MAP: MMType = MMType(0x601C);
    pub const CM_BRG_INFO: MMType = MMType(0x6020);
    pub const CM_CONN_NEW: MMType = MMType(0x6024);
    pub const CM_CONN_REL: MMType = MMType(0x6028);
    pub const CM_CONN_MOD: MMType = MMType(0x602C);
    pub const CM_CONN_INFO: MMType = MMType(0x6030);
    pub const CM_STA_CAP: MMType = MMType(0x6034);
    pub const CM_NW_INFO: MMType = MMType(0x6038);
    pub const CM_GET_BEACON: MMType = MMType(0x603C);
    pub const CM_HFID: MMType = MMType(0x6040);
    pub const CM_MME_ERROR: MMType = MMType(0x6044);
    pub const CM_NW_STATS: MMType = MMType(0x6048);
    pub const CM_LINK_STATS: MMType = MMType(0x604C);
    pub const CM_ROUTE_INFO: MMType = MMType(0x6050);
    pub const CM_UNREACHABLE: MMType = MMType(0x6054);
    pub const CM_MH_CONN_NEW: MMType = MMType(0x6058);
    pub const CM_EXTENDEDTONEMASK: MMType = MMType(0x605C);
    pub const CM_STA_IDENTIFY: MMType = MMType(0x6060);
    pub const CM_TRIGGER_ATTEN_CHAR: MMType = MMType(0x6064);
    pub const CM_START_ATTEN_CHAR: MMType = MMType(0x6068);
    pub const CM_ATTEN_CHAR: MMType = MMType(0x606C);
    pub const CM_PKCS_CERT: MMType = MMType(0x6070);
    pub const CM_MNBC_SOUND: MMType = MMType(0x6074);
    pub const CM_VALIDATE: MMType = MMType(0x6078);
    pub const CM_SLAC_MATCH: MMType = MMType(0x607C);
    pub const CM_SLAC_USER_DATA: MMType = MMType(0x6080);
    pub const CM_ATTEN_PROFILE: MMType = MMType(0x6084);

    pub const fn from_le_bytes(value: [u8; 2]) -> Self {
        Self(u16::from_le_bytes(value))
    }
    pub fn base(&self) -> Self {
        Self(self.0 & !0b11)
    }
    pub fn code(&self) -> MMTypeCode {
        match self.0 & 0b11 {
            0b00 => MMTypeCode::REQ,
            0b01 => MMTypeCode::CNF,
            0b10 => MMTypeCode::IND,
            0b11 => MMTypeCode::RSP,
            _ => unreachable!(),
        }
    }
    pub const fn req(&self) -> Self {
        Self((self.0 & !0b11) + 0b00)
    }
    pub const fn cnf(&self) -> Self {
        Self((self.0 & !0b11) + 0b01)
    }
    pub const fn ind(&self) -> Self {
        Self((self.0 & !0b11) + 0b10)
    }
    pub const fn rsp(&self) -> Self {
        Self((self.0 & !0b11) + 0b11)
    }
    pub const fn value(&self) -> u16 {
        self.0
    }
    pub const fn to_le_bytes(&self) -> [u8; 2] {
        self.0.to_le_bytes()
    }
    pub const fn is_vendor(&self) -> bool {
        (self.0 >> 13) & 0b111 == 0b101
    }
}
impl core::fmt::Debug for MMType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.base() {
            Self::CC_CCO_APPOINT => write!(f, "CC_CCO_APPOINT"),
            Self::CC_BACKUP_APPOINT => write!(f, "CC_BACKUP_APPOINT"),
            Self::CC_LINK_INFO => write!(f, "CC_LINK_INFO"),
            Self::CC_HANDOVER => write!(f, "CC_HANDOVER"),
            Self::CC_HANDOVER_INFO => write!(f, "CC_HANDOVER_INFO"),
            Self::CC_DISCOVER_LIST => write!(f, "CC_DISCOVER_LIST"),
            Self::CC_LINK_NEW => write!(f, "CC_LINK_NEW"),
            Self::CC_LINK_MOD => write!(f, "CC_LINK_MOD"),
            Self::CC_LINK_SQZ => write!(f, "CC_LINK_SQZ"),
            Self::CC_LINK_REL => write!(f, "CC_LINK_REL"),
            Self::CC_DETECT_REPORT => write!(f, "CC_DETECT_REPORT"),
            Self::CC_WHO_RU => write!(f, "CC_WHO_RU"),
            Self::CC_ASSOC => write!(f, "CC_ASSOC"),
            Self::CC_LEAVE => write!(f, "CC_LEAVE"),
            Self::CC_SET_TEI_MAP => write!(f, "CC_SET_TEI_MAP"),
            Self::CC_RELAY => write!(f, "CC_RELAY"),
            Self::CC_BEACON_RELIABILITY => write!(f, "CC_BEACON_RELIABILITY"),
            Self::CC_ALLOC_MOVE => write!(f, "CC_ALLOC_MOVE"),
            Self::CC_ACCESS_NEW => write!(f, "CC_ACCESS_NEW"),
            Self::CC_ACCESS_REL => write!(f, "CC_ACCESS_REL"),
            Self::CC_DCPPC => write!(f, "CC_DCPPC"),
            Self::CC_HP1_DET => write!(f, "CC_HP1_DET"),
            Self::CC_BLE_UPDATE => write!(f, "CC_BLE_UPDATE"),
            Self::CC_BCAST_REPEAT => write!(f, "CC_BCAST_REPEAT"),
            Self::CC_MH_LINK_NEW => write!(f, "CC_MH_LINK_NEW"),
            Self::CC_ISP_DETECTION_REPORT => write!(f, "CC_ISP_DETECTION_REPORT"),
            Self::CC_ISP_START_RESYNC => write!(f, "CC_ISP_START_RESYNC"),
            Self::CC_ISP_FINISH_RESYNC => write!(f, "CC_ISP_FINISH_RESYNC"),
            Self::CC_ISP_RESYNC_DETECTED => write!(f, "CC_ISP_RESYNC_DETECTED"),
            Self::CC_ISP_RESYNC_TRANSMIT => write!(f, "CC_ISP_RESYNC_TRANSMIT"),
            Self::CC_POWERSAVE => write!(f, "CC_POWERSAVE"),
            Self::CC_POWERSAVE_EXIT => write!(f, "CC_POWERSAVE_EXIT"),
            Self::CC_POWERSAVE_LIST => write!(f, "CC_POWERSAVE_LIST"),
            Self::CC_STOP_POWERSAVE => write!(f, "CC_STOP_POWERSAVE"),
            Self::CP_PROXY_APPOINT => write!(f, "CP_PROXY_APPOINT"),
            Self::PH_PROXY_APPOINT => write!(f, "PH_PROXY_APPOINT"),
            Self::CP_PROXY_WAKE => write!(f, "CP_PROXY_WAKE"),
            Self::NN_INL => write!(f, "NN_INL"),
            Self::NN_NEW_NET => write!(f, "NN_NEW_NET"),
            Self::NN_ADD_ALLOC => write!(f, "NN_ADD_ALLOC"),
            Self::NN_REL_ALLOC => write!(f, "NN_REL_ALLOC"),
            Self::NN_REL_NET => write!(f, "NN_REL_NET"),
            Self::CM_UNASSOCIATED_STA => write!(f, "CM_UNASSOCIATED_STA"),
            Self::CM_ENCRYPTED_PAYLOAD => write!(f, "CM_ENCRYPTED_PAYLOAD"),
            Self::CM_SET_KEY => write!(f, "CM_SET_KEY"),
            Self::CM_GET_KEY => write!(f, "CM_GET_KEY"),
            Self::CM_SC_JOIN => write!(f, "CM_SC_JOIN"),
            Self::CM_CHAN_EST => write!(f, "CM_CHAN_EST"),
            Self::CM_TM_UPDATE => write!(f, "CM_TM_UPDATE"),
            Self::CM_AMP_MAP => write!(f, "CM_AMP_MAP"),
            Self::CM_BRG_INFO => write!(f, "CM_BRG_INFO"),
            Self::CM_CONN_NEW => write!(f, "CM_CONN_NEW"),
            Self::CM_CONN_REL => write!(f, "CM_CONN_REL"),
            Self::CM_CONN_MOD => write!(f, "CM_CONN_MOD"),
            Self::CM_CONN_INFO => write!(f, "CM_CONN_INFO"),
            Self::CM_STA_CAP => write!(f, "CM_STA_CAP"),
            Self::CM_NW_INFO => write!(f, "CM_NW_INFO"),
            Self::CM_GET_BEACON => write!(f, "CM_GET_BEACON"),
            Self::CM_HFID => write!(f, "CM_HFID"),
            Self::CM_MME_ERROR => write!(f, "CM_MME_ERROR"),
            Self::CM_NW_STATS => write!(f, "CM_NW_STATS"),
            Self::CM_LINK_STATS => write!(f, "CM_LINK_STATS"),
            Self::CM_ROUTE_INFO => write!(f, "CM_ROUTE_INFO"),
            Self::CM_UNREACHABLE => write!(f, "CM_UNREACHABLE"),
            Self::CM_MH_CONN_NEW => write!(f, "CM_MH_CONN_NEW"),
            Self::CM_EXTENDEDTONEMASK => write!(f, "CM_EXTENDEDTONEMASK"),
            Self::CM_STA_IDENTIFY => write!(f, "CM_STA_IDENTIFY"),
            Self::CM_TRIGGER_ATTEN_CHAR => write!(f, "CM_TRIGGER_ATTEN_CHAR"),
            Self::CM_START_ATTEN_CHAR => write!(f, "CM_START_ATTEN_CHAR"),
            Self::CM_ATTEN_CHAR => write!(f, "CM_ATTEN_CHAR"),
            Self::CM_PKCS_CERT => write!(f, "CM_PKCS_CERT"),
            Self::CM_MNBC_SOUND => write!(f, "CM_MNBC_SOUND"),
            Self::CM_VALIDATE => write!(f, "CM_VALIDATE"),
            Self::CM_SLAC_MATCH => write!(f, "CM_SLAC_MATCH"),
            Self::CM_SLAC_USER_DATA => write!(f, "CM_SLAC_USER_DATA"),
            Self::CM_ATTEN_PROFILE => write!(f, "CM_ATTEN_PROFILE"),
            _ => match (self.0 >> 13) & 0b111 {
                0b000 => write!(f, "STA<>CCo{:04x}", self.0),
                0b001 => write!(f, "Proxy{:04x}", self.0),
                0b010 => write!(f, "CCo<>CCo{:04x}", self.0),
                0b011 => write!(f, "STA<>STA{:04x}", self.0),
                0b100 => write!(f, "Manufacturer{:04x}", self.0),
                0b101 => write!(f, "Vendor{:04x}", self.0),
                _ => write!(f, "Unknown"),
            },
        }?;
        write!(f, ".{:?}", self.code())
    }
}
