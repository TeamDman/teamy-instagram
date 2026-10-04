use super::ReferenceResolution;
use facet::Facet;

/// One URL occurrence in the original UTF-8 text.
///
/// Offsets are bytes, end is exclusive, and ordinal is zero-based among all URL
/// occurrences including unrelated links. Repeated links remain distinct.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct LocatedReference {
    pub start: usize,
    pub end: usize,
    pub ordinal: usize,
    pub resolution: ReferenceResolution,
}
