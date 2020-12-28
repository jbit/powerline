use super::*;
use crate::*;
use core::ops::Deref;

#[derive(Eq, PartialEq, Hash)]
pub struct BridgeInfo<T: Deref<Target = [u8]>>(pub EtherAddr, pub T);
impl<T: Deref<Target = [u8]>> BridgeInfo<T> {
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
impl<T: Deref<Target = [u8]>> Message for BridgeInfo<T> {
    fn message_data(&self) -> &[u8] {
        &self.1
    }
}
impl<T: Deref<Target = [u8]>> core::fmt::Debug for BridgeInfo<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.is_bridge() {
            writeln!(f, "[{:?}] Bridge TEI={}", self.0, self.tei())?;
            for addr in self.destinations() {
                writeln!(f, "  {:?}", addr)?;
            }
        } else {
            writeln!(f, "[{:?}] Not a bridge", self.0)?;
        }
        Ok(())
    }
}
