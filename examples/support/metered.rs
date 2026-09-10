use ic_stable_structures::{Memory, VectorMemory};
use std::{cell::Cell, rc::Rc};

#[derive(Clone, Copy, Debug, Default)]
pub struct Counts {
    pub reads: u64,
    pub read_bytes: u64,
    pub writes: u64,
    pub write_bytes: u64,
    pub grows: u64,
    pub grown_pages: u64,
}

#[derive(Clone, Default)]
pub struct Metered {
    pub bytes: VectorMemory,
    counts: Rc<Cell<Counts>>,
}

impl Metered {
    pub fn reset(&self) {
        self.counts.set(Counts::default());
    }
    pub fn counts(&self) -> Counts {
        self.counts.get()
    }
}

impl Memory for Metered {
    fn size(&self) -> u64 {
        self.bytes.size()
    }
    fn grow(&self, pages: u64) -> i64 {
        let result = self.bytes.grow(pages);
        let mut counts = self.counts.get();
        counts.grows += 1;
        if result >= 0 {
            counts.grown_pages += pages;
        }
        self.counts.set(counts);
        result
    }
    fn read(&self, offset: u64, dst: &mut [u8]) {
        let mut counts = self.counts.get();
        counts.reads += 1;
        counts.read_bytes += dst.len() as u64;
        self.counts.set(counts);
        self.bytes.read(offset, dst);
    }
    fn write(&self, offset: u64, src: &[u8]) {
        let mut counts = self.counts.get();
        counts.writes += 1;
        counts.write_bytes += src.len() as u64;
        self.counts.set(counts);
        self.bytes.write(offset, src);
    }
}
