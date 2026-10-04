use crate::reels::ReelReference;
use facet::Facet;

/// One normalized identity with indices of every retained evidence occurrence.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ReelEvidenceGroup {
    pub reel: ReelReference,
    pub evidence_indices: Vec<usize>,
}
