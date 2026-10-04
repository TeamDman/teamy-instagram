use crate::archive::ArchiveEntryReport;
use crate::archive::ArchiveEntryStatus;
use facet::Facet;

/// Coverage for the reader-visible archive entries, rather than a full-export claim.
#[derive(Facet, Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    /// True only when every reader-visible non-directory entry was typed and parsed.
    /// This does not assert that every optional export section was selected.
    pub all_exposed_entries_parsed: bool,
    pub entry_count: usize,
    pub parsed_count: usize,
    pub parse_failure_count: usize,
    pub unsupported_count: usize,
    pub media_count: usize,
    pub directory_count: usize,
    pub inspected_bytes: u64,
    pub entries: Vec<ArchiveEntryReport>,
}

impl ValidationReport {
    /// Unsupported entries and missing optional sections do not fail validation.
    #[must_use]
    pub const fn has_failures(&self) -> bool {
        self.parse_failure_count != 0
    }

    pub(crate) fn push(&mut self, entry: ArchiveEntryReport) {
        self.entry_count += 1;
        match entry.status {
            ArchiveEntryStatus::Parsed => self.parsed_count += 1,
            ArchiveEntryStatus::ParseFailure => self.parse_failure_count += 1,
            ArchiveEntryStatus::Unsupported => self.unsupported_count += 1,
            ArchiveEntryStatus::Media => self.media_count += 1,
            ArchiveEntryStatus::Directory => self.directory_count += 1,
        }
        self.entries.push(entry);
    }
}
