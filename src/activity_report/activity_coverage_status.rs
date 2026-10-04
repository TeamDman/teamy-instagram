use facet::Facet;

/// Coverage of a particular requested activity within the supported sources.
#[derive(Facet, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ActivityCoverageStatus {
    Present,
    Absent,
    Unsupported,
    Ambiguous,
}
