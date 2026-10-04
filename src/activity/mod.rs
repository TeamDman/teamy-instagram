//! Source-backed reel activity from the selected typed export schemas.
//! Extraction preserves repeated occurrences, and direction or reaction
//! ownership requires explicit exporter identity mapping.

mod activity_kind;
mod activity_source_kind;
mod entry_activities;
mod evidence_provenance;
mod extract_entry;
mod reaction_kind;
mod reel_activity;
mod reference_counts;
mod reference_source;
mod source_timestamp;
mod source_timestamp_unit;

pub use activity_kind::ActivityKind;
pub use activity_source_kind::ActivitySourceKind;
pub use entry_activities::EntryActivities;
pub use evidence_provenance::EvidenceProvenance;
pub use extract_entry::MAX_ACTIVITIES_PER_ENTRY;
pub use extract_entry::extract_entry;
pub use reaction_kind::ReactionKind;
pub use reel_activity::ReelActivity;
pub use reference_counts::ReferenceCounts;
pub use source_timestamp::SourceTimestamp;
pub use source_timestamp_unit::SourceTimestampUnit;
