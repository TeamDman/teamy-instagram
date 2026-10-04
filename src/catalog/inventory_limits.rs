/// Filesystem traversal bounds, independent of archive entry-read bounds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryLimits {
    pub recursive: bool,
    pub max_entries: usize,
    pub max_depth: usize,
}

impl Default for InventoryLimits {
    fn default() -> Self {
        Self {
            recursive: true,
            max_entries: 100_000,
            max_depth: 64,
        }
    }
}
