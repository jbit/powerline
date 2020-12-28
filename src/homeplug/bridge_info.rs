use super::*;
use crate::*;

#[derive(Eq, PartialEq, Hash)]
pub struct BridgeInfo<'a>(pub &'a [u8]);
impl BridgeInfo<'_> {
    pub fn is_bridge(&self) -> bool {
        self.payload()[0] != 0
    }
    pub fn tei(&self) -> u8 {
        self.payload()[1]
    }
    pub fn destinations(&self) -> impl ExactSizeIterator + Iterator<Item = EtherAddr> + '_ {
        let count = if self.is_bridge() {
            self.payload()[2] as usize
        } else {
            0
        };
        self.payload()[3..]
            .chunks_exact(6)
            .take(count as usize)
            .map(|data| EtherAddr::from_slice(data))
    }
}
impl Message for BridgeInfo<'_> {
    fn message_data(&self) -> &[u8] {
        &self.0
    }
}
impl core::fmt::Debug for BridgeInfo<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.is_bridge() {
            write!(f, "Bridge(TEI={} ", self.tei())?;
            let mut l = f.debug_list();
            l.entries(self.destinations());
            l.finish()?;
            write!(f, ")")?;
        } else {
            write!(f, "Not a bridge")?;
        }
        Ok(())
    }
}
