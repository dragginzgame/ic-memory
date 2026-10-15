//! Bounded serialization buffers; refused output is discarded before publication.

///
/// BoundedWriter
///
/// One buffer for a single bounded serialization attempt. Callers supply an
/// admitted prefix and capacity hint; an error discards the entire attempt.
/// The attempted length retains the Cell boundary's rejected-size evidence.
///

pub struct BoundedWriter {
    pub(crate) bytes: Vec<u8>,
    pub(crate) attempted_len: usize,
    limit: usize,
}

impl BoundedWriter {
    pub(crate) const fn new(bytes: Vec<u8>, limit: usize) -> Self {
        Self {
            attempted_len: bytes.len(),
            bytes,
            limit,
        }
    }
}

impl std::io::Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.write_all(bytes)?;
        Ok(bytes.len())
    }

    // Record the rejected size for the Cell boundary's typed failure. No
    // append or capacity reservation occurs until the complete slice fits.
    #[inline]
    fn write_all(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.attempted_len = self
            .attempted_len
            .checked_add(bytes.len())
            .ok_or_else(|| std::io::Error::other("encoded record length overflow"))?;
        if self.attempted_len > self.limit {
            return Err(std::io::ErrorKind::WriteZero.into());
        }
        if self.attempted_len > self.bytes.capacity() {
            let capacity = self
                .bytes
                .capacity()
                .saturating_mul(2)
                .max(self.attempted_len)
                .min(self.limit);
            self.bytes.reserve_exact(capacity - self.bytes.len());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
