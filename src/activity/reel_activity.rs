use super::ActivityKind;
use super::ActivitySourceKind;
use super::EvidenceProvenance;
use super::ReactionKind;
use super::SourceTimestamp;
use crate::reels::ReelReference;
use facet::Facet;

/// One action linked to one source URL occurrence. Repeated references remain
/// separate evidence; callers may group by reel identity without losing them.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ReelActivity {
    pub activity: ActivityKind,
    pub source_kind: ActivitySourceKind,
    pub reel: ReelReference,
    pub provenance: EvidenceProvenance,
    /// Activity or reaction timestamp; absent reaction timestamps stay absent.
    pub timestamp: Option<SourceTimestamp>,
    /// For message evidence this remains available independently of a reaction
    /// timestamp, and is never substituted for an unknown reaction time.
    pub message_timestamp: Option<SourceTimestamp>,
    pub reaction_kind: Option<ReactionKind>,
}
