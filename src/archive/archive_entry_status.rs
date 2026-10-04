use facet::Facet;

/// An entry's validation outcome, independent of whether optional sections exist.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ArchiveEntryStatus {
    Parsed,
    ParseFailure,
    Unsupported,
    Media,
    Directory,
}
