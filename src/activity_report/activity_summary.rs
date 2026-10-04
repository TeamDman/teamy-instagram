use super::ActivityCoverage;
use crate::activity::ReferenceCounts;
use crate::archive::ValidationReport;
use facet::Facet;

/// Aggregate output deliberately excludes reel IDs, URLs, labels and messages.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ActivitySummary {
    pub unique_reel_count: usize,
    pub activity_occurrence_count: usize,
    pub incidental_reference_count: usize,
    pub identity_configured: bool,
    /// Export view metadata cannot establish a complete reel watch history.
    pub complete_watch_history: bool,
    pub complete_export_coverage: bool,
    pub coverage: Vec<ActivityCoverage>,
    pub reference_counts: ReferenceCounts,
    pub unresolved_shared_messages: usize,
    pub ambiguous_shared_messages: usize,
    pub unresolved_reactions: usize,
    pub ambiguous_reactions: usize,
    pub excluded_other_reactions: usize,
    pub validation: ValidationReport,
}
