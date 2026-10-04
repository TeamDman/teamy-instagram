use super::ActivityCoverageStatus;
use crate::activity::ActivityKind;
use facet::Facet;

/// Activity coverage remains separate from strict JSON/schema validation.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ActivityCoverage {
    pub activity: ActivityKind,
    pub status: ActivityCoverageStatus,
    pub occurrence_count: usize,
    pub distinct_reel_count: usize,
    /// Export records, messages or reactions, grouped by source indices.
    /// Multiple URL representations do not inflate this source-event count.
    pub source_event_count: usize,
    /// Missing optional files produce zero here, without a parse failure.
    pub source_entry_count: usize,
    pub parsed_source_entry_count: usize,
    pub failed_source_entry_count: usize,
    pub ambiguous_reference_count: usize,
    /// Counts source messages/reactions once, independently of URL occurrences.
    pub unresolved_event_count: usize,
}
