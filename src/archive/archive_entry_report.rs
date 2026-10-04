use crate::archive::ArchiveEntryCategory;
use crate::archive::ArchiveEntryErrorCode;
use crate::archive::ArchiveEntryStatus;
use crate::models::ParsedEntrySummary;
use facet::Facet;

/// A privacy-safe entry report keyed only by its ZIP reader index.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntryReport {
    pub index: usize,
    pub category: ArchiveEntryCategory,
    pub status: ArchiveEntryStatus,
    pub error_code: Option<ArchiveEntryErrorCode>,
    pub summary: Option<ParsedEntrySummary>,
}
