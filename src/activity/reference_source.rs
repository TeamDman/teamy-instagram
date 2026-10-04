use super::ActivityKind;
use super::ActivitySourceKind;
use super::EvidenceProvenance;
use super::SourceTimestamp;

/// Private context shared by all URL occurrences within one source field.
#[derive(Debug, Clone)]
pub(super) struct ReferenceSource {
    pub activity: ActivityKind,
    pub source_kind: ActivitySourceKind,
    pub provenance: EvidenceProvenance,
    pub timestamp: Option<SourceTimestamp>,
    pub message_timestamp: Option<SourceTimestamp>,
}
