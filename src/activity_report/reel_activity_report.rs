use super::ActivitySummary;
use super::ReelEvidenceGroup;
use crate::activity::ReelActivity;
use facet::Facet;

/// Detailed local activity output. Its IDs and URLs are private user data.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct ReelActivityReport {
    pub summary: ActivitySummary,
    pub reels: Vec<ReelEvidenceGroup>,
    pub activities: Vec<ReelActivity>,
    /// Caption links are references, without an action claim about that reel.
    pub incidental_references: Vec<ReelActivity>,
}

impl ReelActivityReport {
    #[must_use]
    pub const fn has_failures(&self) -> bool {
        self.summary.validation.has_failures()
    }

    /// Return only fixed categories and aggregate counts for private validation.
    #[must_use]
    pub fn aggregate_summary(&self) -> ActivitySummary {
        self.summary.clone()
    }
}
