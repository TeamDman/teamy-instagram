use facet::Facet;

/// Public failure codes deliberately omit parser diagnostics and private values.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchiveEntryErrorCode {
    EntryUnavailable,
    EntryTooLarge,
    TotalReadLimit,
    EntryReadFailure,
    InvalidJson,
    SchemaMismatch,
}
