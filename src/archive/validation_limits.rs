/// Bounds applied before and during decompression of recognized JSON entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationLimits {
    pub max_entry_bytes: u64,
    pub max_total_read_bytes: u64,
    pub max_entries: usize,
}

impl Default for ValidationLimits {
    fn default() -> Self {
        Self {
            max_entry_bytes: 32 * 1024 * 1024,
            max_total_read_bytes: 512 * 1024 * 1024,
            max_entries: 100_000,
        }
    }
}
