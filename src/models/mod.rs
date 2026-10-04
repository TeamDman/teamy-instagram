//! Strict raw models for the selected Instagram export sections.
//!
//! These types preserve source-defined information. A message reaction does not
//! imply liking a reel, and sender names do not establish exporter direction.
//! Raw model values are private data; validation reports expose only counts and
//! fixed failure codes.

mod empty_export_object;
mod export_label_value;
mod export_record;
mod liked_comment;
mod liked_comments;
mod message;
mod message_joinable_mode;
mod message_media;
mod message_participant;
mod message_reaction;
mod message_share;
mod message_thread;
mod parsed_entry_summary;
mod strict_json;
mod string_list_data;

use crate::archive::ArchiveEntryCategory;
use crate::archive::ArchiveEntryErrorCode;
pub use empty_export_object::EmptyExportObject;
pub use export_label_value::ExportLabelValue;
pub use export_record::ExportRecord;
use facet::Facet;
use facet_format::DeserializeErrorKind;
pub use liked_comment::LikedComment;
pub use liked_comments::LikedComments;
pub use message::Message;
pub use message_joinable_mode::MessageJoinableMode;
pub use message_media::MessageMedia;
pub use message_participant::MessageParticipant;
pub use message_reaction::MessageReaction;
pub use message_share::MessageShare;
pub use message_thread::MessageThread;
pub use parsed_entry_summary::ParsedEntrySummary;
pub use string_list_data::StringListData;

/// Select a known schema from a service-defined relative section path.
///
/// A ZIP may wrap its contents in a root directory. The root directory name is
/// irrelevant and is never retained. Other filenames remain unsupported.
#[must_use]
pub fn classify_entry(name: &str) -> Option<ArchiveEntryCategory> {
    const ADS_ROOT: &str = "ads_information/";
    const ACTIVITY_ROOT: &str = "your_instagram_activity/";
    let ads_relative = name.strip_prefix(ADS_ROOT).or_else(|| {
        name.split_once("/ads_information/")
            .map(|(_, relative)| relative)
    });
    if ads_relative == Some("ads_and_topics/videos_watched.json") {
        return Some(ArchiveEntryCategory::AdWatchedVideos);
    }
    let relative = name.strip_prefix(ACTIVITY_ROOT).or_else(|| {
        name.split_once("/your_instagram_activity/")
            .map(|(_, relative)| relative)
    })?;
    match relative {
        "saved/saved_posts.json" => Some(ArchiveEntryCategory::SavedPosts),
        "likes/liked_posts.json" => Some(ArchiveEntryCategory::LikedPosts),
        "likes/liked_comments.json" => Some(ArchiveEntryCategory::LikedComments),
        "story_interactions/stories_viewed.json" => Some(ArchiveEntryCategory::StoriesViewed),
        _ => classify_message_thread(relative),
    }
}

fn classify_message_thread(relative: &str) -> Option<ArchiveEntryCategory> {
    let relative = relative.strip_prefix("messages/")?;
    let mut components = relative.split('/');
    if !matches!(components.next(), Some("inbox" | "message_requests")) {
        return None;
    }
    let thread = components.next()?;
    if thread.is_empty() {
        return None;
    }
    let file = components.next()?;
    if components.next().is_some() {
        return None;
    }
    let index = file.strip_prefix("message_")?.strip_suffix(".json")?;
    if index.is_empty() || !index.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some(ArchiveEntryCategory::MessageThread)
}

/// Parse a selected entry with strict nested field checking in every build.
///
/// Arrays observed only empty remain unsupported when nonempty. Errors are
/// reduced to fixed codes without rendering Facet diagnostics, field names,
/// source values, or source excerpts.
///
/// # Errors
///
/// Returns a sanitized code for invalid JSON, a schema mismatch, or an
/// unsupported category passed by a caller.
pub fn validate_json(
    category: ArchiveEntryCategory,
    bytes: &[u8],
) -> Result<ParsedEntrySummary, ArchiveEntryErrorCode> {
    match category {
        ArchiveEntryCategory::SavedPosts
        | ArchiveEntryCategory::LikedPosts
        | ArchiveEntryCategory::StoriesViewed
        | ArchiveEntryCategory::AdWatchedVideos => {
            let records: Vec<ExportRecord> = parse_json(bytes)?;
            if records.iter().any(|record| {
                !record.media.is_empty()
                    || record
                        .label_values
                        .iter()
                        .any(ExportLabelValue::has_unsupported_vec)
            }) {
                return Err(ArchiveEntryErrorCode::SchemaMismatch);
            }
            Ok(ParsedEntrySummary {
                record_count: count(records.len())?,
                ..ParsedEntrySummary::default()
            })
        }
        ArchiveEntryCategory::LikedComments => {
            let comments: LikedComments = parse_json(bytes)?;
            Ok(ParsedEntrySummary {
                record_count: count(comments.likes_comment_likes.len())?,
                ..ParsedEntrySummary::default()
            })
        }
        ArchiveEntryCategory::MessageThread => {
            let thread: MessageThread = parse_json(bytes)?;
            if !thread.magic_words.is_empty()
                || thread.messages.iter().any(|message| {
                    message.share.as_ref().is_some_and(|share| {
                        share.media.as_ref().is_some_and(|media| !media.is_empty())
                    })
                })
            {
                return Err(ArchiveEntryErrorCode::SchemaMismatch);
            }
            let reaction_count = thread
                .messages
                .iter()
                .filter_map(|message| message.reactions.as_ref())
                .map(Vec::len)
                .try_fold(0_u64, |total, length| {
                    total
                        .checked_add(count(length)?)
                        .ok_or(ArchiveEntryErrorCode::SchemaMismatch)
                })?;
            Ok(ParsedEntrySummary {
                record_count: 0,
                message_count: count(thread.messages.len())?,
                reaction_count,
            })
        }
        _ => Err(ArchiveEntryErrorCode::SchemaMismatch),
    }
}

fn count(length: usize) -> Result<u64, ArchiveEntryErrorCode> {
    length
        .try_into()
        .map_err(|_error| ArchiveEntryErrorCode::SchemaMismatch)
}

pub(crate) fn parse_json<T: Facet<'static>>(bytes: &[u8]) -> Result<T, ArchiveEntryErrorCode> {
    strict_json::check::<T>(bytes)?;
    facet_json::from_slice(bytes).map_err(|error| sanitized_parse_error(&error))
}

fn sanitized_parse_error(error: &facet_json::DeserializeError) -> ArchiveEntryErrorCode {
    match error.kind {
        DeserializeErrorKind::UnexpectedChar { .. }
        | DeserializeErrorKind::UnexpectedEof { .. }
        | DeserializeErrorKind::InvalidUtf8 { .. } => ArchiveEntryErrorCode::InvalidJson,
        _ => ArchiveEntryErrorCode::SchemaMismatch,
    }
}
