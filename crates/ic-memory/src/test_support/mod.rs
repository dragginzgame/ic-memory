//! Opt-in native heap-memory observations for tests, never IC performance evidence.
//!
//! Enable `test-support` to observe calls through this crate's exact `Memory`
//! trait. Clones share bytes and counters; reset changes counters only. Direct
//! access to `Metered::bytes` bypasses observation. No timers or canister hooks.

use std::{cell::Cell, rc::Rc};

use crate::ic_stable_structures::{Memory, VectorMemory};

///
/// Counts
///
/// Native memory calls and their supplied byte/page quantities since reset.
///
/// Values clip at `u64::MAX`; `overflowed` records any lost count or quantity.
/// Read/write calls are counted before delegation. Growth is counted after its
/// return, including failure; only successful growth contributes pages.
///

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Counts {
    /// Read calls.
    pub reads: u64,
    /// Bytes supplied to reads.
    pub read_bytes: u64,
    /// Write calls.
    pub writes: u64,
    /// Bytes supplied to writes.
    pub write_bytes: u64,
    /// Growth calls that returned.
    pub grows: u64,
    /// Pages accepted by successful growth calls.
    pub grown_pages: u64,
    /// Whether any count or quantity exceeded its numeric cap.
    pub overflowed: bool,
}

const fn add(value: &mut u64, overflowed: &mut bool, amount: u64) {
    if let Some(next) = value.checked_add(amount) {
        *value = next;
    } else {
        *value = u64::MAX;
        *overflowed = true;
    }
}

///
/// Metered
///
/// A native `VectorMemory` backend with shared call observations.
///
/// Clones share both storage and the counter window. The delegated substrate
/// retains its own growth/error behavior; this wrapper observes native calls,
/// not stable-memory charges, timings, instructions or cycles.
///

#[derive(Clone, Default)]
pub struct Metered {
    /// Heap-backed bytes; direct operations bypass the observation counters.
    pub bytes: VectorMemory,
    counts: Rc<Cell<Counts>>,
}

impl Metered {
    /// Reset the shared counter window without changing bytes or page capacity.
    pub fn reset(&self) {
        self.counts.set(Counts::default());
    }

    /// Copy the current shared observations, including actual counter overflow.
    #[must_use]
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
        add(&mut counts.grows, &mut counts.overflowed, 1);
        if result >= 0 {
            add(&mut counts.grown_pages, &mut counts.overflowed, pages);
        }
        self.counts.set(counts);
        result
    }

    fn read(&self, offset: u64, dst: &mut [u8]) {
        let mut counts = self.counts.get();
        add(&mut counts.reads, &mut counts.overflowed, 1);
        add(
            &mut counts.read_bytes,
            &mut counts.overflowed,
            dst.len() as u64,
        );
        self.counts.set(counts);
        self.bytes.read(offset, dst);
    }

    fn write(&self, offset: u64, src: &[u8]) {
        let mut counts = self.counts.get();
        add(&mut counts.writes, &mut counts.overflowed, 1);
        add(
            &mut counts.write_bytes,
            &mut counts.overflowed,
            src.len() as u64,
        );
        self.counts.set(counts);
        self.bytes.write(offset, src);
    }
}

#[cfg(test)]
mod tests;
