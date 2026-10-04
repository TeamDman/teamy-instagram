use super::ActivityKind;
use super::ActivitySourceKind;
use super::EntryActivities;
use super::EvidenceProvenance;
use super::ReactionKind;
use super::ReelActivity;
use super::SourceTimestamp;
use super::SourceTimestampUnit;
use super::reference_source::ReferenceSource;
use crate::archive::ArchiveEntryCategory;
use crate::archive::ArchiveEntryErrorCode;
use crate::identity::ExporterIdentity;
use crate::identity::IdentityMatch;
use crate::models::ExportLabelValue;
use crate::models::ExportRecord;
use crate::models::Message;
use crate::models::MessageParticipant;
use crate::models::MessageThread;
use crate::models::ParsedEntrySummary;
use crate::models::parse_json;
use crate::reels::ReferenceResolution;
use crate::reels::UnsupportedInstagramReference;
use crate::reels::find_references;

/// Limits expansion when many URL occurrences and reactions share a message.
/// The archive reader separately bounds compressed entry reads.
pub const MAX_ACTIVITIES_PER_ENTRY: usize = 100_000;

/// Strictly parse a supported entry and retain each reel reference occurrence.
/// Message direction and reaction ownership require explicit exporter labels;
/// raw names, message contents, and arbitrary source labels are never returned.
///
/// # Errors
///
/// Returns a fixed, sanitized error for an unsupported category, invalid JSON,
/// schema mismatch, or an evidence expansion beyond the per-entry limit.
pub fn extract_entry(
    category: ArchiveEntryCategory,
    bytes: &[u8],
    entry_index: usize,
    identity: &ExporterIdentity,
) -> Result<EntryActivities, ArchiveEntryErrorCode> {
    match category {
        ArchiveEntryCategory::SavedPosts
        | ArchiveEntryCategory::LikedPosts
        | ArchiveEntryCategory::StoriesViewed
        | ArchiveEntryCategory::AdWatchedVideos => extract_records(category, bytes, entry_index),
        ArchiveEntryCategory::MessageThread => extract_thread(bytes, entry_index, identity),
        _ => Err(ArchiveEntryErrorCode::SchemaMismatch),
    }
}

fn extract_records(
    category: ArchiveEntryCategory,
    bytes: &[u8],
    entry_index: usize,
) -> Result<EntryActivities, ArchiveEntryErrorCode> {
    let records: Vec<ExportRecord> = parse_json(bytes)?;
    if records.iter().any(|record| !record.media.is_empty()) {
        return Err(ArchiveEntryErrorCode::SchemaMismatch);
    }
    let (activity, source_kind) = match category {
        ArchiveEntryCategory::SavedPosts => {
            (ActivityKind::Bookmarked, ActivitySourceKind::SavedPost)
        }
        ArchiveEntryCategory::LikedPosts => (ActivityKind::Liked, ActivitySourceKind::LikedPost),
        ArchiveEntryCategory::StoriesViewed => {
            (ActivityKind::Seen, ActivitySourceKind::ViewedStory)
        }
        ArchiveEntryCategory::AdWatchedVideos => {
            (ActivityKind::Seen, ActivitySourceKind::AdWatchedVideo)
        }
        _ => return Err(ArchiveEntryErrorCode::SchemaMismatch),
    };
    let mut output = EntryActivities::new(category);
    output.summary = ParsedEntrySummary {
        record_count: count(records.len())?,
        ..ParsedEntrySummary::default()
    };
    for (record_index, record) in records.iter().enumerate() {
        let source = ReferenceSource {
            activity,
            source_kind,
            provenance: EvidenceProvenance {
                entry_index,
                entry_category: category,
                record_index: Some(record_index),
                message_index: None,
                reaction_index: None,
                field_path: format!("[{record_index}].label_values"),
                occurrence_index: 0,
                field_byte_start: 0,
                field_byte_end: 0,
            },
            timestamp: Some(SourceTimestamp {
                value: record.timestamp,
                unit: SourceTimestampUnit::Seconds,
            }),
            message_timestamp: None,
        };
        extract_labels(&record.label_values, &source, &mut output)?;
    }
    Ok(output)
}

fn extract_labels(
    labels: &[ExportLabelValue],
    source: &ReferenceSource,
    output: &mut EntryActivities,
) -> Result<(), ArchiveEntryErrorCode> {
    for (label_index, label) in labels.iter().enumerate() {
        if label.vec.as_ref().is_some_and(|items| !items.is_empty()) {
            return Err(ArchiveEntryErrorCode::SchemaMismatch);
        }
        let mut label_source = source.clone();
        if label.label.as_deref() == Some("Caption") {
            label_source.activity = ActivityKind::IncidentalReference;
            label_source.source_kind = ActivitySourceKind::RecordCaptionLink;
        }
        let prefix = format!("{}[{label_index}]", label_source.provenance.field_path);
        for (field, value) in [("href", &label.href), ("value", &label.value)] {
            if let Some(value) = value {
                let mut field_source = label_source.clone();
                field_source.provenance.field_path = format!("{prefix}.{field}");
                extract_references(value, &field_source, output)?;
            }
        }
        if let Some(children) = &label.dict {
            let mut child_source = label_source.clone();
            child_source.provenance.field_path = format!("{prefix}.dict");
            extract_labels(children, &child_source, output)?;
        }
    }
    Ok(())
}

fn extract_thread(
    bytes: &[u8],
    entry_index: usize,
    identity: &ExporterIdentity,
) -> Result<EntryActivities, ArchiveEntryErrorCode> {
    let thread: MessageThread = parse_json(bytes)?;
    if !thread.magic_words.is_empty()
        || thread.messages.iter().any(|message| {
            message
                .share
                .as_ref()
                .is_some_and(|share| share.media.as_ref().is_some_and(|media| !media.is_empty()))
        })
    {
        return Err(ArchiveEntryErrorCode::SchemaMismatch);
    }
    let reaction_count = thread
        .messages
        .iter()
        .filter_map(|message| message.reactions.as_ref())
        .try_fold(0_u64, |total, reactions| {
            total
                .checked_add(count(reactions.len())?)
                .ok_or(ArchiveEntryErrorCode::SchemaMismatch)
        })?;
    let mut output = EntryActivities::new(ArchiveEntryCategory::MessageThread);
    output.summary = ParsedEntrySummary {
        record_count: 0,
        message_count: count(thread.messages.len())?,
        reaction_count,
    };
    for (message_index, message) in thread.messages.iter().enumerate() {
        extract_message(
            message,
            message_index,
            entry_index,
            &thread.participants,
            identity,
            &mut output,
        )?;
    }
    Ok(output)
}

fn extract_message(
    message: &Message,
    message_index: usize,
    entry_index: usize,
    participants: &[MessageParticipant],
    identity: &ExporterIdentity,
    output: &mut EntryActivities,
) -> Result<(), ArchiveEntryErrorCode> {
    let sender = identity.resolve_label(&message.sender_name, participants);
    let activity = match sender {
        IdentityMatch::SelfActor => ActivityKind::Sent,
        IdentityMatch::OtherActor => ActivityKind::Received,
        IdentityMatch::Unresolved | IdentityMatch::Ambiguous => ActivityKind::SharedUnresolved,
    };
    let message_timestamp = SourceTimestamp {
        value: message.timestamp_ms,
        unit: SourceTimestampUnit::Milliseconds,
    };
    let mut source = ReferenceSource {
        activity,
        source_kind: ActivitySourceKind::MessageSharedLink,
        provenance: EvidenceProvenance {
            entry_index,
            entry_category: ArchiveEntryCategory::MessageThread,
            record_index: None,
            message_index: Some(message_index),
            reaction_index: None,
            field_path: format!("messages[{message_index}].share.link"),
            occurrence_index: 0,
            field_byte_start: 0,
            field_byte_end: 0,
        },
        timestamp: Some(message_timestamp),
        message_timestamp: Some(message_timestamp),
    };
    let first_activity = output.activities.len();
    let first_ambiguous_reference = output.reference_counts.ambiguous_reel_occurrences;
    if let Some(link) = message.share.as_ref().and_then(|share| share.link.as_ref()) {
        extract_references(link, &source, output)?;
    }
    if let Some(content) = &message.content {
        source.source_kind = ActivitySourceKind::MessageContentLink;
        source.provenance.field_path = format!("messages[{message_index}].content");
        extract_references(content, &source, output)?;
    }
    let references = output.activities[first_activity..].to_vec();
    let ambiguous_references =
        output.reference_counts.ambiguous_reel_occurrences - first_ambiguous_reference;
    if references.is_empty() && ambiguous_references == 0 {
        return Ok(());
    }
    attribute_ambiguous_direction(sender, ambiguous_references, output)?;
    match sender {
        IdentityMatch::Unresolved => output.unresolved_shared_messages += 1,
        IdentityMatch::Ambiguous => output.ambiguous_shared_messages += 1,
        IdentityMatch::SelfActor | IdentityMatch::OtherActor => {}
    }
    extract_reactions(
        message,
        &references,
        ambiguous_references,
        participants,
        identity,
        output,
    )
}

fn attribute_ambiguous_direction(
    sender: IdentityMatch,
    count: usize,
    output: &mut EntryActivities,
) -> Result<(), ArchiveEntryErrorCode> {
    match sender {
        IdentityMatch::SelfActor => {
            add_ambiguous_count(&mut output.ambiguous_sent_references, count)
        }
        IdentityMatch::OtherActor => {
            add_ambiguous_count(&mut output.ambiguous_received_references, count)
        }
        IdentityMatch::Unresolved | IdentityMatch::Ambiguous => {
            add_ambiguous_count(&mut output.ambiguous_sent_references, count)?;
            add_ambiguous_count(&mut output.ambiguous_received_references, count)
        }
    }
}

fn add_ambiguous_count(total: &mut usize, count: usize) -> Result<(), ArchiveEntryErrorCode> {
    *total = total
        .checked_add(count)
        .ok_or(ArchiveEntryErrorCode::ActivityLimitExceeded)?;
    Ok(())
}
fn extract_reactions(
    message: &Message,
    references: &[ReelActivity],
    ambiguous_references: usize,
    participants: &[MessageParticipant],
    identity: &ExporterIdentity,
    output: &mut EntryActivities,
) -> Result<(), ArchiveEntryErrorCode> {
    let Some(reactions) = &message.reactions else {
        return Ok(());
    };
    for (reaction_index, reaction) in reactions.iter().enumerate() {
        let activity = match identity.resolve_label(&reaction.actor, participants) {
            IdentityMatch::SelfActor => ActivityKind::ReactedTo,
            IdentityMatch::OtherActor => {
                output.excluded_other_reactions += 1;
                continue;
            }
            IdentityMatch::Unresolved => {
                output.unresolved_reactions += 1;
                ActivityKind::ReactionUnresolved
            }
            IdentityMatch::Ambiguous => {
                output.ambiguous_reactions += 1;
                ActivityKind::ReactionUnresolved
            }
        };
        add_ambiguous_count(
            &mut output.ambiguous_reacted_references,
            ambiguous_references,
        )?;
        for reference in references {
            let mut event = reference.clone();
            event.activity = activity;
            event.provenance.reaction_index = Some(reaction_index);
            event.timestamp = reaction.timestamp.map(|value| SourceTimestamp {
                value,
                unit: SourceTimestampUnit::Unspecified,
            });
            event.reaction_kind = Some(reaction_kind(&reaction.reaction));
            push_activity(output, event)?;
        }
    }
    Ok(())
}

fn extract_references(
    value: &str,
    source: &ReferenceSource,
    output: &mut EntryActivities,
) -> Result<(), ArchiveEntryErrorCode> {
    for located in find_references(value) {
        let reel = match located.resolution {
            ReferenceResolution::Reel(reel) => {
                output.reference_counts.recognized_reel_occurrences += 1;
                reel
            }
            ReferenceResolution::Unsupported(kind) => {
                output.reference_counts.unsupported_instagram_occurrences += 1;
                if matches!(
                    kind,
                    UnsupportedInstagramReference::ShareReel
                        | UnsupportedInstagramReference::InvalidReel
                        | UnsupportedInstagramReference::InvalidUrl
                ) {
                    output.reference_counts.ambiguous_reel_occurrences += 1;
                }
                continue;
            }
            ReferenceResolution::Unrelated => {
                output.reference_counts.unrelated_url_occurrences += 1;
                continue;
            }
        };
        let mut provenance = source.provenance.clone();
        provenance.occurrence_index = located.ordinal;
        provenance.field_byte_start = located.start;
        provenance.field_byte_end = located.end;
        push_activity(
            output,
            ReelActivity {
                activity: source.activity,
                source_kind: source.source_kind,
                reel,
                provenance,
                timestamp: source.timestamp,
                message_timestamp: source.message_timestamp,
                reaction_kind: None,
            },
        )?;
    }
    Ok(())
}

fn push_activity(
    output: &mut EntryActivities,
    event: ReelActivity,
) -> Result<(), ArchiveEntryErrorCode> {
    if output.activities.len() >= MAX_ACTIVITIES_PER_ENTRY {
        return Err(ArchiveEntryErrorCode::ActivityLimitExceeded);
    }
    output.activities.push(event);
    Ok(())
}

fn reaction_kind(token: &str) -> ReactionKind {
    if matches!(
        token,
        "\u{2764}"
            | "\u{2764}\u{fe0f}"
            | "heart"
            | "\u{00e2}\u{009d}\u{00a4}\u{00ef}\u{00b8}\u{008f}"
    ) {
        ReactionKind::Heart
    } else {
        ReactionKind::Other
    }
}

fn count(length: usize) -> Result<u64, ArchiveEntryErrorCode> {
    length
        .try_into()
        .map_err(|_error| ArchiveEntryErrorCode::SchemaMismatch)
}
