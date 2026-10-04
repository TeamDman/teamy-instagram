use super::ReelActivity;
use super::ReferenceCounts;
use crate::archive::ArchiveEntryCategory;
use crate::models::ParsedEntrySummary;
use facet::Facet;

/// Extracted evidence and finite coverage counters for one strictly parsed
/// entry. Unresolved counters count source messages/reactions once, even when
/// the same source contains several reel URL occurrences.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct EntryActivities {
    pub category: ArchiveEntryCategory,
    pub activities: Vec<ReelActivity>,
    pub summary: ParsedEntrySummary,
    pub reference_counts: ReferenceCounts,
    /// Reel-like URL occurrences whose identity is unresolved, attributed by
    /// the explicitly mapped sender or to both directions if sender is unknown.
    pub ambiguous_sent_references: usize,
    pub ambiguous_received_references: usize,
    /// Reel-like URL occurrences per self or unresolved reaction. Other actors'
    /// reactions and messages without reactions do not contribute.
    pub ambiguous_reacted_references: usize,
    pub unresolved_shared_messages: usize,
    pub ambiguous_shared_messages: usize,
    pub unresolved_reactions: usize,
    pub ambiguous_reactions: usize,
    pub excluded_other_reactions: usize,
}

impl EntryActivities {
    #[must_use]
    pub fn new(category: ArchiveEntryCategory) -> Self {
        Self {
            category,
            activities: Vec::new(),
            summary: ParsedEntrySummary::default(),
            reference_counts: ReferenceCounts::default(),
            ambiguous_sent_references: 0,
            ambiguous_received_references: 0,
            ambiguous_reacted_references: 0,
            unresolved_shared_messages: 0,
            ambiguous_shared_messages: 0,
            unresolved_reactions: 0,
            ambiguous_reactions: 0,
            excluded_other_reactions: 0,
        }
    }
}
