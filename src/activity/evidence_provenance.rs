use crate::archive::ArchiveEntryCategory;
use facet::Facet;

/// A stable location within this archive reader's ordered entry inventory.
/// Paths use only fixed schema field names and numeric array indices. Byte
/// offsets identify a URL occurrence within the decoded source field value;
/// they are not offsets in compressed bytes or the complete JSON entry.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct EvidenceProvenance {
    pub entry_index: usize,
    pub entry_category: ArchiveEntryCategory,
    pub record_index: Option<usize>,
    pub message_index: Option<usize>,
    pub reaction_index: Option<usize>,
    pub field_path: String,
    pub occurrence_index: usize,
    pub field_byte_start: usize,
    pub field_byte_end: usize,
}
