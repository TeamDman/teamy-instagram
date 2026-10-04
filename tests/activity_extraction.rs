use teamy_instagram::activity::ActivityKind;
use teamy_instagram::activity::ActivitySourceKind;
use teamy_instagram::activity::EntryActivities;
use teamy_instagram::activity::ReactionKind;
use teamy_instagram::activity::SourceTimestamp;
use teamy_instagram::activity::SourceTimestampUnit;
use teamy_instagram::activity::extract_entry;
use teamy_instagram::archive::ArchiveEntryCategory;
use teamy_instagram::archive::ArchiveEntryErrorCode;
use teamy_instagram::identity::ExporterIdentity;
use teamy_instagram::models::ParsedEntrySummary;

// These fixtures are entirely synthetic. Public tests must never include
// export-derived names, identifiers, links, messages, paths, or timestamps.
const ENTRY_INDEX: usize = 17;
const PARTICIPANTS: &str = r#"{"name":"Synthetic exporter"},{"name":"Synthetic peer"}"#;
const REEL: &str = "https://www.instagram.com/reel/Synthetic_A1/";
const LABEL: &str = r#"{"href":"https://www.instagram.com/reel/Synthetic_A1/"}"#;

fn identity() -> ExporterIdentity {
    ExporterIdentity::new(vec!["Synthetic exporter".to_owned()])
        .expect("synthetic explicit identity is valid")
}

fn records(labels: &str) -> String {
    format!(
        r#"[{{"fbid":"synthetic-record","timestamp":123,"media":[],"label_values":[{labels}]}}]"#
    )
}

fn thread(messages: &str) -> String {
    thread_with_participants(PARTICIPANTS, messages)
}

fn thread_with_participants(participants: &str, messages: &str) -> String {
    format!(
        r#"{{"participants":[{participants}],"messages":[{messages}],"title":"Synthetic thread","is_still_participant":true,"thread_path":"synthetic_thread","magic_words":[]}}"#
    )
}

fn message(sender: &str, timestamp: u64, fields: &str) -> String {
    format!(
        r#"{{"sender_name":"{sender}","timestamp_ms":{timestamp},"is_geoblocked_for_viewer":false{fields}}}"#
    )
}

fn extract(
    category: ArchiveEntryCategory,
    data: &str,
    identity: &ExporterIdentity,
) -> EntryActivities {
    extract_entry(category, data.as_bytes(), ENTRY_INDEX, identity)
        .expect("synthetic supported entry extracts")
}

fn count_activity(entry: &EntryActivities, activity: ActivityKind) -> usize {
    entry
        .activities
        .iter()
        .filter(|event| event.activity == activity)
        .count()
}

#[test]
fn saved_liked_and_source_defined_seen_remain_distinct() {
    let data = records(LABEL);
    for (category, activity, source_kind) in [
        (
            ArchiveEntryCategory::SavedPosts,
            ActivityKind::Bookmarked,
            ActivitySourceKind::SavedPost,
        ),
        (
            ArchiveEntryCategory::LikedPosts,
            ActivityKind::Liked,
            ActivitySourceKind::LikedPost,
        ),
        (
            ArchiveEntryCategory::StoriesViewed,
            ActivityKind::Seen,
            ActivitySourceKind::ViewedStory,
        ),
        (
            ArchiveEntryCategory::AdWatchedVideos,
            ActivityKind::Seen,
            ActivitySourceKind::AdWatchedVideo,
        ),
    ] {
        let result = extract(category, &data, &ExporterIdentity::default());
        assert_eq!(result.activities.len(), 1);
        assert_eq!(result.reference_counts.recognized_reel_occurrences, 1);
        assert_eq!(
            result.summary,
            ParsedEntrySummary {
                record_count: 1,
                message_count: 0,
                reaction_count: 0,
            }
        );
        let event = &result.activities[0];
        assert_eq!(event.activity, activity);
        assert_eq!(event.source_kind, source_kind);
        assert_eq!(event.reel.reel_id.as_str(), "Synthetic_A1");
        assert_eq!(event.reel.canonical_url, REEL);
        assert_eq!(event.provenance.entry_index, ENTRY_INDEX);
        assert_eq!(event.provenance.entry_category, category);
        assert_eq!(event.provenance.record_index, Some(0));
        assert_eq!(event.provenance.message_index, None);
        assert_eq!(event.provenance.reaction_index, None);
        assert_eq!(event.provenance.field_path, "[0].label_values[0].href");
        assert_eq!(event.provenance.field_byte_start, 0);
        assert_eq!(event.provenance.field_byte_end, REEL.len());
        assert_eq!(
            event.timestamp,
            Some(SourceTimestamp {
                value: 123,
                unit: SourceTimestampUnit::Seconds,
            })
        );
        assert_eq!(event.message_timestamp, None);
        assert_eq!(event.reaction_kind, None);
    }
}

#[test]
fn explicit_identity_supplies_sent_received_and_self_reaction_categories() {
    let outgoing = message(
        "Synthetic exporter",
        123_000,
        r#", "share":{"link":"https://www.instagram.com/reel/Synthetic_A1/"}"#,
    );
    let incoming = message(
        "Synthetic peer",
        124_000,
        r#", "share":{"link":"https://www.instagram.com/reel/Synthetic_B2/"},"reactions":[{"reaction":"heart","actor":"Synthetic exporter","timestamp":125},{"reaction":"heart","actor":"Synthetic peer","timestamp":126}]"#,
    );
    let result = extract(
        ArchiveEntryCategory::MessageThread,
        &thread(&format!("{outgoing},{incoming}")),
        &identity(),
    );
    assert_eq!(count_activity(&result, ActivityKind::Sent), 1);
    assert_eq!(count_activity(&result, ActivityKind::Received), 1);
    assert_eq!(count_activity(&result, ActivityKind::ReactedTo), 1);
    assert_eq!(count_activity(&result, ActivityKind::Liked), 0);
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 2);
    assert_eq!(result.excluded_other_reactions, 1);
    assert_eq!(result.summary.message_count, 2);
    assert_eq!(result.summary.reaction_count, 2);
    let reaction = &result.activities[2];
    assert_eq!(reaction.reel.reel_id.as_str(), "Synthetic_B2");
    assert_eq!(reaction.reaction_kind, Some(ReactionKind::Heart));
    assert_eq!(reaction.provenance.message_index, Some(1));
    assert_eq!(reaction.provenance.reaction_index, Some(0));
    assert_eq!(reaction.provenance.field_path, "messages[1].share.link");
    assert_eq!(
        reaction.timestamp,
        Some(SourceTimestamp {
            value: 125,
            unit: SourceTimestampUnit::Unspecified,
        })
    );
    assert_eq!(
        reaction.message_timestamp,
        Some(SourceTimestamp {
            value: 124_000,
            unit: SourceTimestampUnit::Milliseconds,
        })
    );
}

#[test]
fn recursive_href_and_value_fields_preserve_repeated_occurrences() {
    let repeated_value = format!("Synthetic prefix {REEL} then {REEL}");
    let labels = format!(
        r#"{{"href":"{REEL}","value":"{repeated_value}","dict":[{{"title":"Synthetic nested","dict":[{{"href":"{REEL}","value":"{REEL}"}}]}}]}}"#
    );
    let result = extract(
        ArchiveEntryCategory::SavedPosts,
        &records(&labels),
        &identity(),
    );
    assert_eq!(result.activities.len(), 5);
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 5);
    assert!(result.activities.iter().all(|event| {
        event.activity == ActivityKind::Bookmarked
            && event.reel.reel_id.as_str() == "Synthetic_A1"
            && event.provenance.record_index == Some(0)
    }));
    let paths = result
        .activities
        .iter()
        .map(|event| event.provenance.field_path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            "[0].label_values[0].href",
            "[0].label_values[0].value",
            "[0].label_values[0].value",
            "[0].label_values[0].dict[0].dict[0].href",
            "[0].label_values[0].dict[0].dict[0].value",
        ]
    );
    for (event, ordinal) in result.activities[1..3].iter().zip([0, 1]) {
        assert_eq!(event.provenance.occurrence_index, ordinal);
        assert_eq!(
            &repeated_value[event.provenance.field_byte_start..event.provenance.field_byte_end],
            REEL
        );
    }
    assert_ne!(
        result.activities[1].provenance.field_byte_start,
        result.activities[2].provenance.field_byte_start
    );
}

#[test]
fn repeated_reel_across_records_retains_both_timestamps_and_locations() {
    let record = r#"{"fbid":"synthetic-record","timestamp":123,"media":[],"label_values":[{"href":"https://www.instagram.com/reel/Synthetic_A1/"}]}"#;
    let later = record.replace("123", "456");
    let data = format!("[{record},{later}]");
    let result = extract(ArchiveEntryCategory::LikedPosts, &data, &identity());
    assert_eq!(result.activities.len(), 2);
    assert_eq!(result.summary.record_count, 2);
    assert_eq!(result.activities[0].reel, result.activities[1].reel);
    assert_eq!(result.activities[0].provenance.record_index, Some(0));
    assert_eq!(result.activities[1].provenance.record_index, Some(1));
    assert_eq!(
        result.activities[0].timestamp.map(|stamp| stamp.value),
        Some(123)
    );
    assert_eq!(
        result.activities[1].timestamp.map(|stamp| stamp.value),
        Some(456)
    );
}

#[test]
fn share_link_and_content_have_separate_evidence_for_same_reel() {
    let data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "share":{"link":"https://www.instagram.com/reel/Synthetic_A1/?synthetic=1"},"content":"Synthetic link https://instagram.com/reels/Synthetic_A1/#synthetic", "reactions":[{"reaction":"heart","actor":"Synthetic exporter"}]"#,
    ));
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &identity());
    assert_eq!(result.activities.len(), 4);
    assert_eq!(count_activity(&result, ActivityKind::Received), 2);
    assert_eq!(count_activity(&result, ActivityKind::ReactedTo), 2);
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 2);
    assert_eq!(
        result.activities[0].source_kind,
        ActivitySourceKind::MessageSharedLink
    );
    assert_eq!(
        result.activities[1].source_kind,
        ActivitySourceKind::MessageContentLink
    );
    assert_eq!(
        result.activities[0].provenance.field_path,
        "messages[0].share.link"
    );
    assert_eq!(
        result.activities[1].provenance.field_path,
        "messages[0].content"
    );
    assert!(
        result
            .activities
            .iter()
            .all(|event| event.reel.canonical_url == REEL)
    );
    for reaction in &result.activities[2..] {
        assert_eq!(reaction.timestamp, None);
        assert_eq!(reaction.provenance.reaction_index, Some(0));
        assert_eq!(
            reaction.message_timestamp.map(|stamp| stamp.value),
            Some(123_000)
        );
    }
}

#[test]
fn absent_exporter_mapping_keeps_direction_and_all_reaction_owners_unresolved() {
    let data = thread(&message(
        "Synthetic exporter",
        123_000,
        r#", "content":"https://www.instagram.com/reel/Synthetic_A1/ https://www.instagram.com/reel/Synthetic_A1/", "reactions":[{"reaction":"heart","actor":"Synthetic exporter"},{"reaction":"heart","actor":"Synthetic peer"}]"#,
    ));
    let result = extract(
        ArchiveEntryCategory::MessageThread,
        &data,
        &ExporterIdentity::default(),
    );
    assert_eq!(count_activity(&result, ActivityKind::SharedUnresolved), 2);
    assert_eq!(count_activity(&result, ActivityKind::ReactionUnresolved), 4);
    assert_eq!(count_activity(&result, ActivityKind::Sent), 0);
    assert_eq!(count_activity(&result, ActivityKind::Received), 0);
    assert_eq!(count_activity(&result, ActivityKind::ReactedTo), 0);
    assert_eq!(result.unresolved_shared_messages, 1);
    assert_eq!(result.unresolved_reactions, 2);
    assert_eq!(result.excluded_other_reactions, 0);
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 2);
}

#[test]
fn duplicate_self_participant_labels_are_ambiguous_not_self() {
    let participants =
        r#"{"name":"Synthetic exporter"},{"name":"Synthetic exporter"},{"name":"Synthetic peer"}"#;
    let messages = message(
        "Synthetic exporter",
        123_000,
        r#", "share":{"link":"https://www.instagram.com/reel/Synthetic_A1/"}, "reactions":[{"reaction":"heart","actor":"Synthetic exporter"}]"#,
    );
    let result = extract(
        ArchiveEntryCategory::MessageThread,
        &thread_with_participants(participants, &messages),
        &identity(),
    );
    assert_eq!(count_activity(&result, ActivityKind::SharedUnresolved), 1);
    assert_eq!(count_activity(&result, ActivityKind::ReactionUnresolved), 1);
    assert_eq!(count_activity(&result, ActivityKind::ReactedTo), 0);
    assert_eq!(result.ambiguous_shared_messages, 1);
    assert_eq!(result.ambiguous_reactions, 1);
    assert_eq!(result.unresolved_shared_messages, 0);
    assert_eq!(result.unresolved_reactions, 0);
}

#[test]
fn exporter_labels_are_case_sensitive_and_missing_mapped_labels_do_not_guess() {
    let data = thread(&message(
        "Synthetic exporter",
        123_000,
        r#", "share":{"link":"https://www.instagram.com/reel/Synthetic_A1/"}, "reactions":[{"reaction":"heart","actor":"Synthetic exporter"}]"#,
    ));
    let wrong_case = ExporterIdentity::new(vec!["synthetic exporter".to_owned()])
        .expect("synthetic mapping can be configured even when unmatched");
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &wrong_case);
    assert_eq!(count_activity(&result, ActivityKind::SharedUnresolved), 1);
    assert_eq!(count_activity(&result, ActivityKind::ReactionUnresolved), 1);
    assert_eq!(count_activity(&result, ActivityKind::ReactedTo), 0);
    let only_peer = thread_with_participants(
        r#"{"name":"Synthetic peer"}"#,
        &message(
            "Synthetic peer",
            123_000,
            r#", "content":"https://www.instagram.com/reel/Synthetic_A1/""#,
        ),
    );
    let result = extract(ArchiveEntryCategory::MessageThread, &only_peer, &identity());
    assert_eq!(count_activity(&result, ActivityKind::SharedUnresolved), 1);
    assert_eq!(count_activity(&result, ActivityKind::Received), 0);
}

#[test]
fn explicitly_configured_aliases_resolve_each_matching_thread() {
    let aliases = ExporterIdentity::new(vec![
        "Synthetic exporter".to_owned(),
        "Synthetic alternate".to_owned(),
    ])
    .expect("synthetic aliases are explicit");
    for self_label in ["Synthetic exporter", "Synthetic alternate"] {
        let participants = format!(r#"{{"name":"{self_label}"}},{{"name":"Synthetic peer"}}"#);
        let data = thread_with_participants(
            &participants,
            &message(
                self_label,
                123_000,
                r#", "content":"https://www.instagram.com/reel/Synthetic_A1/""#,
            ),
        );
        let result = extract(ArchiveEntryCategory::MessageThread, &data, &aliases);
        assert_eq!(count_activity(&result, ActivityKind::Sent), 1);
        assert_eq!(result.unresolved_shared_messages, 0);
    }
}

#[test]
fn other_actor_reactions_are_excluded_without_inflating_occurrence_count() {
    let data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "content":"https://www.instagram.com/reel/Synthetic_A1/ https://www.instagram.com/reel/Synthetic_A1/", "reactions":[{"reaction":"heart","actor":"Synthetic peer"}]"#,
    ));
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &identity());
    assert_eq!(result.activities.len(), 2);
    assert_eq!(count_activity(&result, ActivityKind::Received), 2);
    assert_eq!(count_activity(&result, ActivityKind::ReactedTo), 0);
    assert_eq!(result.excluded_other_reactions, 1);
    assert_eq!(result.summary.reaction_count, 1);
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 2);
}

#[test]
fn heart_variants_and_other_reactions_are_separate_from_reel_likes() {
    let data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "content":"https://www.instagram.com/reel/Synthetic_A1/", "reactions":[{"reaction":"heart","actor":"Synthetic exporter"},{"reaction":"\u2764","actor":"Synthetic exporter"},{"reaction":"\u2764\ufe0f","actor":"Synthetic exporter"},{"reaction":"\u00e2\u009d\u00a4\u00ef\u00b8\u008f","actor":"Synthetic exporter"},{"reaction":"synthetic-other","actor":"Synthetic exporter"}]"#,
    ));
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &identity());
    assert_eq!(count_activity(&result, ActivityKind::ReactedTo), 5);
    assert_eq!(count_activity(&result, ActivityKind::Liked), 0);
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 1);
    assert_eq!(
        result
            .activities
            .iter()
            .filter(|event| event.reaction_kind == Some(ReactionKind::Heart))
            .count(),
        4
    );
    assert_eq!(
        result.activities[5].reaction_kind,
        Some(ReactionKind::Other)
    );
    assert_eq!(result.activities[5].provenance.reaction_index, Some(4));
}

#[test]
fn absent_reaction_time_stays_absent_and_present_time_has_unspecified_unit() {
    let data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "content":"https://www.instagram.com/reel/Synthetic_A1/", "reactions":[{"reaction":"heart","actor":"Synthetic exporter"},{"reaction":"heart","actor":"Synthetic exporter","timestamp":456}]"#,
    ));
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &identity());
    assert_eq!(result.activities[1].timestamp, None);
    assert_eq!(
        result.activities[2].timestamp,
        Some(SourceTimestamp {
            value: 456,
            unit: SourceTimestampUnit::Unspecified
        })
    );
    assert_eq!(
        result.activities[1].message_timestamp,
        Some(SourceTimestamp {
            value: 123_000,
            unit: SourceTimestampUnit::Milliseconds
        })
    );
    assert_eq!(
        result.activities[1].message_timestamp,
        result.activities[2].message_timestamp
    );
}

#[test]
fn optional_message_sections_and_empty_selected_data_are_successful() {
    for category in [
        ArchiveEntryCategory::SavedPosts,
        ArchiveEntryCategory::LikedPosts,
        ArchiveEntryCategory::StoriesViewed,
        ArchiveEntryCategory::AdWatchedVideos,
    ] {
        let result = extract(category, "[]", &identity());
        assert_eq!(result.activities, []);
        assert_eq!(result.summary, ParsedEntrySummary::default());
        assert_eq!(result.reference_counts.recognized_reel_occurrences, 0);
    }
    let data = thread(&message("Synthetic exporter", 123_000, ""));
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &identity());
    assert_eq!(result.activities, []);
    assert_eq!(result.summary.message_count, 1);
    assert_eq!(result.summary.reaction_count, 0);
    let empty = extract(
        ArchiveEntryCategory::MessageThread,
        &thread(""),
        &identity(),
    );
    assert_eq!(empty.summary, ParsedEntrySummary::default());
}

#[test]
fn messages_without_reel_references_do_not_create_reaction_evidence() {
    let data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "content":"Synthetic ordinary message", "reactions":[{"reaction":"heart","actor":"Synthetic exporter"}]"#,
    ));
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &identity());
    assert_eq!(result.activities, []);
    assert_eq!(result.summary.reaction_count, 1);
    assert_eq!(result.unresolved_reactions, 0);
    assert_eq!(result.ambiguous_reactions, 0);
}

#[test]
fn opaque_share_links_and_non_reel_posts_have_distinct_ambiguity_counts() {
    let data = records(
        r#"{"value":"https://www.instagram.com/share/reel/SyntheticOpaque/ https://www.instagram.com/p/Synthetic_A1/ https://www.instagram.com/stories/synthetic/ https://example.invalid/reel/Synthetic_A1/ https://www.instagram.com/reel/Synthetic_A1/"}"#,
    );
    let result = extract(ArchiveEntryCategory::SavedPosts, &data, &identity());
    assert_eq!(result.activities.len(), 1);
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 1);
    assert_eq!(result.reference_counts.unsupported_instagram_occurrences, 3);
    assert_eq!(result.reference_counts.ambiguous_reel_occurrences, 1);
    assert_eq!(result.reference_counts.unrelated_url_occurrences, 1);
    assert_eq!(result.activities[0].provenance.occurrence_index, 4);
}

#[test]
fn only_reference_fields_contribute_evidence_not_labels_or_message_share_text() {
    let data = records(
        r#"{"label":"https://www.instagram.com/reel/Synthetic_A1/","title":"https://www.instagram.com/reel/Synthetic_A1/"}"#,
    );
    let result = extract(ArchiveEntryCategory::LikedPosts, &data, &identity());
    assert_eq!(result.activities, []);
    let data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "share":{"share_text":"https://www.instagram.com/reel/Synthetic_A1/","original_content_owner":"https://www.instagram.com/reel/Synthetic_A1/"}"#,
    ));
    let result = extract(ArchiveEntryCategory::MessageThread, &data, &identity());
    assert_eq!(result.activities, []);
}

#[test]
fn extraction_retains_strict_numeric_types_and_rejects_trailing_json() {
    let data = records(LABEL);
    let wrong_type = data.replace(r#""timestamp":123"#, r#""timestamp":"123""#);
    assert_eq!(
        extract_entry(
            ArchiveEntryCategory::SavedPosts,
            wrong_type.as_bytes(),
            ENTRY_INDEX,
            &identity()
        ),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
    let message_data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "content":"https://www.instagram.com/reel/Synthetic_A1/", "reactions":[{"reaction":"heart","actor":"Synthetic exporter","timestamp":"456"}]"#,
    ));
    assert_eq!(
        extract_entry(
            ArchiveEntryCategory::MessageThread,
            message_data.as_bytes(),
            ENTRY_INDEX,
            &identity()
        ),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
    for suffix in [" false", " []", "synthetic"] {
        let invalid = format!("{data}{suffix}");
        assert_eq!(
            extract_entry(
                ArchiveEntryCategory::SavedPosts,
                invalid.as_bytes(),
                ENTRY_INDEX,
                &identity()
            ),
            Err(ArchiveEntryErrorCode::InvalidJson)
        );
    }
}

#[test]
fn unknown_nested_fields_and_nonempty_unknown_media_remain_failures() {
    let data = records(
        r#"{"href":"https://www.instagram.com/reel/Synthetic_A1/","synthetic_unknown":true}"#,
    );
    assert_eq!(
        extract_entry(
            ArchiveEntryCategory::SavedPosts,
            data.as_bytes(),
            ENTRY_INDEX,
            &identity()
        ),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
    let data = records(LABEL).replace(r#""media":[]"#, r#""media":[{}]"#);
    assert_eq!(
        extract_entry(
            ArchiveEntryCategory::SavedPosts,
            data.as_bytes(),
            ENTRY_INDEX,
            &identity()
        ),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
    let data = thread(&message(
        "Synthetic peer",
        123_000,
        r#", "share":{"link":"https://www.instagram.com/reel/Synthetic_A1/","media":[{}]}"#,
    ));
    assert_eq!(
        extract_entry(
            ArchiveEntryCategory::MessageThread,
            data.as_bytes(),
            ENTRY_INDEX,
            &identity()
        ),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
}

#[test]
fn unsupported_categories_do_not_silently_become_empty_activity_results() {
    for category in [
        ArchiveEntryCategory::LikedComments,
        ArchiveEntryCategory::UnsupportedJson,
    ] {
        assert_eq!(
            extract_entry(category, b"[]", ENTRY_INDEX, &identity()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}

#[test]
fn reference_and_reaction_expansion_is_bounded_per_entry() {
    let links = std::iter::repeat_n(REEL, 400).collect::<Vec<_>>().join(" ");
    let reactions =
        std::iter::repeat_n(r#"{"reaction":"heart","actor":"Synthetic exporter"}"#, 250)
            .collect::<Vec<_>>()
            .join(",");
    let fields = format!(r#", "content":"{links}","reactions":[{reactions}]"#);
    let data = thread(&message("Synthetic peer", 123_000, &fields));
    assert_eq!(
        extract_entry(
            ArchiveEntryCategory::MessageThread,
            data.as_bytes(),
            ENTRY_INDEX,
            &identity()
        ),
        Err(ArchiveEntryErrorCode::ActivityLimitExceeded)
    );
}

#[test]
fn empty_recursive_vector_fields_parse_but_nonempty_unknown_items_fail() {
    let labels = r#"{"href":"https://www.instagram.com/reel/Synthetic_A1/","vec":[],"dict":[{"value":"https://www.instagram.com/reel/Synthetic_A1/","vec":[]}] }"#;
    let data = records(labels);
    let result = extract(ArchiveEntryCategory::SavedPosts, &data, &identity());
    assert_eq!(result.activities.len(), 2);
    for invalid in [
        data.replacen(r#""vec":[]"#, r#""vec":[{}]"#, 1),
        data.replace(
            r#""value":"https://www.instagram.com/reel/Synthetic_A1/","vec":[]"#,
            r#""value":"https://www.instagram.com/reel/Synthetic_A1/","vec":[{}]"#,
        ),
    ] {
        assert_eq!(
            extract_entry(
                ArchiveEntryCategory::SavedPosts,
                invalid.as_bytes(),
                ENTRY_INDEX,
                &identity()
            ),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}

#[test]
fn caption_links_are_incidental_and_parent_caption_kind_propagates() {
    let data = records(
        r#"{"href":"https://instagram.com/reel/SyntheticPrimary/","value":"https://instagram.com/reel/SyntheticPrimary/?tracking=synthetic"},{"label":"Caption","href":"https://instagram.com/reel/SyntheticCaptionHref/","value":"Synthetic caption https://instagram.com/reel/SyntheticCaptionText/","dict":[{"value":"https://instagram.com/reel/SyntheticCaptionNested/"}]}"#,
    );
    let result = extract(ArchiveEntryCategory::LikedPosts, &data, &identity());
    assert_eq!(count_activity(&result, ActivityKind::Liked), 2);
    assert_eq!(
        count_activity(&result, ActivityKind::IncidentalReference),
        3
    );
    assert_eq!(result.reference_counts.recognized_reel_occurrences, 5);
    assert!(
        result
            .activities
            .iter()
            .filter(|event| event.activity == ActivityKind::Liked)
            .all(|event| {
                event.reel.reel_id.as_str() == "SyntheticPrimary"
                    && event.source_kind == ActivitySourceKind::LikedPost
            })
    );
    let incidental = result
        .activities
        .iter()
        .filter(|event| event.activity == ActivityKind::IncidentalReference)
        .collect::<Vec<_>>();
    assert!(incidental.iter().all(|event| {
        event.source_kind == ActivitySourceKind::RecordCaptionLink
            && event.reel.reel_id.as_str() != "SyntheticPrimary"
            && event.timestamp
                == Some(SourceTimestamp {
                    value: 123,
                    unit: SourceTimestampUnit::Seconds,
                })
            && event.provenance.entry_index == ENTRY_INDEX
            && event.provenance.record_index == Some(0)
    }));
    assert_eq!(
        incidental[0].provenance.field_path,
        "[0].label_values[1].href"
    );
    assert_eq!(
        incidental[1].provenance.field_path,
        "[0].label_values[1].value"
    );
    assert_eq!(
        incidental[2].provenance.field_path,
        "[0].label_values[1].dict[0].value"
    );
    assert_eq!(
        incidental[2].reel.reel_id.as_str(),
        "SyntheticCaptionNested"
    );
}
