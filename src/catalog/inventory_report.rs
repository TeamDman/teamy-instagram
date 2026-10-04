use super::ArchiveRecord;
use facet::Facet;

/// Reader-visible files and explicit traversal limitations. Archive paths are
/// private user output; they must not be copied into public fixtures or logs.
#[derive(Facet, Debug, Clone, Default, PartialEq, Eq)]
pub struct InventoryReport {
    pub archives: Vec<ArchiveRecord>,
    pub scanned_entry_count: u64,
    pub skipped_symlink_count: u64,
    pub unreadable_entry_count: u64,
    pub depth_limited_directory_count: u64,
    pub entry_limit_reached: bool,
}

impl InventoryReport {
    #[must_use]
    pub const fn is_incomplete(&self) -> bool {
        self.unreadable_entry_count > 0
            || self.depth_limited_directory_count > 0
            || self.entry_limit_reached
    }
}
