use super::ExportDate;
use facet::Facet;

/// A file matching the Instagram service ZIP naming convention. Its contents
/// are not opened during inventory; the name alone does not validate a ZIP.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ArchiveRecord {
    pub path: String,
    pub size_bytes: u64,
    pub modified_unix_milliseconds: Option<u64>,
    pub export_date: Option<ExportDate>,
}
