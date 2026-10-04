use super::ActivityCoverage;
use super::ActivityCoverageStatus;
use super::ActivitySummary;
use super::ReelActivityReport;
use super::ReelEvidenceGroup;
use crate::activity::ActivityKind;
use crate::activity::ReelActivity;
use crate::activity::ReferenceCounts;
use crate::activity::{self};
use crate::archive::ArchiveEntryCategory;
use crate::archive::ArchiveEntryErrorCode;
use crate::archive::ArchiveEntryStatus;
use crate::archive::ValidationLimits;
use crate::archive::ValidationReport;
use crate::archive::{self};
use crate::identity::ExporterIdentity;
use crate::models;
use crate::reels::ReelId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use teamy_cancellation::CancellationToken;

const MAX_REPORT_ACTIVITIES: usize = 1_000_000;

/// Extract normalized reel evidence with a single bounded pass over known entries.
///
/// Unsupported entries remain inventoried; missing optional sections remain
/// absent. Direction requires the caller's explicit exporter-label assertions.
/// URL occurrences and source actions are retained, rather than deduplicated.
///
/// # Errors
/// Returns sanitized archive/limit errors or cancellation. Per-entry schema,
/// read and event-count failures remain in the validation report.
pub fn extract_archive(
    path: &Path,
    limits: &ValidationLimits,
    identity: &ExporterIdentity,
    cancellation: &CancellationToken,
) -> eyre::Result<ReelActivityReport> {
    let mut state = ExtractionState::default();
    let validation =
        archive::visit_archive(path, limits, cancellation, &mut |category, index, bytes| {
            if category == ArchiveEntryCategory::LikedComments {
                return models::validate_json(category, bytes);
            }
            let entry = activity::extract_entry(category, bytes, index, identity)?;
            if entry.activities.len()
                > MAX_REPORT_ACTIVITIES
                    .saturating_sub(state.activities.len() + state.incidental_references.len())
            {
                return Err(ArchiveEntryErrorCode::ActivityLimitExceeded);
            }
            let summary = entry.summary;
            state.reference_counts.recognized_reel_occurrences +=
                entry.reference_counts.recognized_reel_occurrences;
            state.reference_counts.unsupported_instagram_occurrences +=
                entry.reference_counts.unsupported_instagram_occurrences;
            state.reference_counts.unrelated_url_occurrences +=
                entry.reference_counts.unrelated_url_occurrences;
            state.reference_counts.ambiguous_reel_occurrences +=
                entry.reference_counts.ambiguous_reel_occurrences;
            state
                .ambiguities
                .push((category, entry.reference_counts.ambiguous_reel_occurrences));
            state.unresolved_shared_messages += entry.unresolved_shared_messages;
            state.ambiguous_shared_messages += entry.ambiguous_shared_messages;
            state.unresolved_reactions += entry.unresolved_reactions;
            state.ambiguous_reactions += entry.ambiguous_reactions;
            state.excluded_other_reactions += entry.excluded_other_reactions;
            state.ambiguous_sent_references += entry.ambiguous_sent_references;
            state.ambiguous_received_references += entry.ambiguous_received_references;
            state.ambiguous_reacted_references += entry.ambiguous_reacted_references;
            for event in entry.activities {
                if event.activity == ActivityKind::IncidentalReference {
                    state.incidental_references.push(event);
                } else {
                    state.activities.push(event);
                }
            }
            Ok(summary)
        })?;
    cancellation.bail_if_cancelled()?;
    let mut reels = BTreeMap::<ReelId, ReelEvidenceGroup>::new();
    for (index, event) in state.activities.iter().enumerate() {
        cancellation.bail_if_cancelled()?;
        reels
            .entry(event.reel.reel_id.clone())
            .or_insert_with(|| ReelEvidenceGroup {
                reel: event.reel.clone(),
                evidence_indices: Vec::new(),
            })
            .evidence_indices
            .push(index);
    }
    let coverage = [
        ActivityKind::Seen,
        ActivityKind::Liked,
        ActivityKind::Bookmarked,
        ActivityKind::Sent,
        ActivityKind::Received,
        ActivityKind::ReactedTo,
    ]
    .into_iter()
    .map(|kind| state.coverage(kind, &validation))
    .collect();
    let summary = ActivitySummary {
        unique_reel_count: reels.len(),
        activity_occurrence_count: state.activities.len(),
        incidental_reference_count: state.incidental_references.len(),
        identity_configured: identity.is_configured(),
        complete_watch_history: false,
        complete_export_coverage: false,
        coverage,
        reference_counts: state.reference_counts,
        unresolved_shared_messages: state.unresolved_shared_messages,
        ambiguous_shared_messages: state.ambiguous_shared_messages,
        unresolved_reactions: state.unresolved_reactions,
        ambiguous_reactions: state.ambiguous_reactions,
        excluded_other_reactions: state.excluded_other_reactions,
        validation,
    };
    Ok(ReelActivityReport {
        summary,
        reels: reels.into_values().collect(),
        activities: state.activities,
        incidental_references: state.incidental_references,
    })
}

#[derive(Debug, Default)]
struct ExtractionState {
    activities: Vec<ReelActivity>,
    incidental_references: Vec<ReelActivity>,
    reference_counts: ReferenceCounts,
    ambiguities: Vec<(ArchiveEntryCategory, usize)>,
    unresolved_shared_messages: usize,
    ambiguous_shared_messages: usize,
    unresolved_reactions: usize,
    ambiguous_reactions: usize,
    excluded_other_reactions: usize,
    ambiguous_sent_references: usize,
    ambiguous_received_references: usize,
    ambiguous_reacted_references: usize,
}

impl ExtractionState {
    fn coverage(&self, kind: ActivityKind, validation: &ValidationReport) -> ActivityCoverage {
        let entries = validation
            .entries
            .iter()
            .filter(|entry| relevant_source(kind, entry.category))
            .collect::<Vec<_>>();
        let source_entry_count = entries.len();
        let parsed_source_entry_count = entries
            .iter()
            .filter(|entry| entry.status == ArchiveEntryStatus::Parsed)
            .count();
        let failed_source_entry_count = entries
            .iter()
            .filter(|entry| entry.status == ArchiveEntryStatus::ParseFailure)
            .count();
        let occurrences = self
            .activities
            .iter()
            .filter(|event| event.activity == kind)
            .collect::<Vec<_>>();
        let occurrence_count = occurrences.len();
        let distinct_reel_count = occurrences
            .iter()
            .map(|event| &event.reel.reel_id)
            .collect::<BTreeSet<_>>()
            .len();
        let source_event_count = occurrences
            .iter()
            .map(|event| {
                let provenance = &event.provenance;
                (
                    provenance.entry_index,
                    provenance.record_index,
                    provenance.message_index,
                    provenance.reaction_index,
                )
            })
            .collect::<BTreeSet<_>>()
            .len();
        let ambiguous_reference_count = self
            .ambiguities
            .iter()
            .filter(|(category, _)| relevant_source(kind, *category))
            .map(|(_, count)| count)
            .sum::<usize>();
        let ambiguous_reference_count = match kind {
            ActivityKind::Sent => self.ambiguous_sent_references,
            ActivityKind::Received => self.ambiguous_received_references,
            ActivityKind::ReactedTo => self.ambiguous_reacted_references,
            _ => ambiguous_reference_count,
        };
        let unresolved_event_count = match kind {
            ActivityKind::Sent | ActivityKind::Received => {
                self.unresolved_shared_messages + self.ambiguous_shared_messages
            }
            ActivityKind::ReactedTo => self.unresolved_reactions + self.ambiguous_reactions,
            _ => 0,
        };
        let status = if failed_source_entry_count > 0 {
            ActivityCoverageStatus::Unsupported
        } else if ambiguous_reference_count > 0 || unresolved_event_count > 0 {
            ActivityCoverageStatus::Ambiguous
        } else if occurrence_count > 0 {
            ActivityCoverageStatus::Present
        } else {
            ActivityCoverageStatus::Absent
        };
        ActivityCoverage {
            activity: kind,
            status,
            occurrence_count,
            distinct_reel_count,
            source_event_count,
            source_entry_count,
            parsed_source_entry_count,
            failed_source_entry_count,
            ambiguous_reference_count,
            unresolved_event_count,
        }
    }
}

fn relevant_source(kind: ActivityKind, category: ArchiveEntryCategory) -> bool {
    match kind {
        ActivityKind::Seen => matches!(
            category,
            ArchiveEntryCategory::StoriesViewed | ArchiveEntryCategory::AdWatchedVideos
        ),
        ActivityKind::Liked => category == ArchiveEntryCategory::LikedPosts,
        ActivityKind::Bookmarked => category == ArchiveEntryCategory::SavedPosts,
        ActivityKind::Sent | ActivityKind::Received | ActivityKind::ReactedTo => {
            category == ArchiveEntryCategory::MessageThread
        }
        ActivityKind::SharedUnresolved
        | ActivityKind::ReactionUnresolved
        | ActivityKind::IncidentalReference => false,
    }
}
