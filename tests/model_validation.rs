use teamy_instagram::archive::ArchiveEntryCategory;
use teamy_instagram::archive::ArchiveEntryErrorCode;
use teamy_instagram::models::ParsedEntrySummary;
use teamy_instagram::models::classify_entry;
use teamy_instagram::models::validate_json;

// Every value in these fixtures is synthetic. No export-derived identifiers,
// content, timestamps, names, or links belong in public tests.
const ACTIVITY: &str = r#"[
    {
        "fbid":"synthetic-id",
        "timestamp":123,
        "media":[],
        "label_values":[
            {"label":"Synthetic label","value":"Synthetic value"},
            {"href":"https://example.invalid/reel/synthetic-id"},
            {"title":"Synthetic container","dict":[
                {"title":"Synthetic nested container","dict":[
                    {"label":"Synthetic leaf","value":"Synthetic nested value"}
                ]},
                {"title":"Synthetic empty container","dict":[]}
            ]},
            {"label":null,"value":null,"href":null,"title":null,"dict":null}
        ]
    }
]"#;

const THREAD: &str = r#"{
    "participants":[{"name":"Synthetic sender"},{"name":"Synthetic recipient"}],
    "messages":[
        {
            "sender_name":"Synthetic sender",
            "timestamp_ms":123000,
            "is_geoblocked_for_viewer":false,
            "content":"Synthetic content",
            "is_unsent_image_by_messenger_kid_parent":false,
            "share":{
                "link":"https://example.invalid/reel/synthetic-id",
                "share_text":"Synthetic share",
                "original_content_owner":"Synthetic owner",
                "media":[],
                "profile_share_name":"Synthetic profile",
                "profile_share_username":"synthetic_profile"
            },
            "reactions":[
                {"reaction":"heart","actor":"Synthetic recipient","timestamp":124},
                {"reaction":"heart","actor":"Synthetic sender"}
            ],
            "photos":[{"uri":"synthetic/photo.jpg","creation_timestamp":120}],
            "videos":[{"uri":"synthetic/video.mp4"}],
            "call_duration":5
        },
        {
            "sender_name":"Synthetic recipient",
            "timestamp_ms":125000,
            "is_geoblocked_for_viewer":false
        }
    ],
    "title":"Synthetic conversation",
    "is_still_participant":true,
    "thread_path":"synthetic_thread",
    "magic_words":[],
    "joinable_mode":{"mode":1,"link":""},
    "is_pending":false
}"#;

const COMMENTS: &str = r#"{
    "likes_comment_likes":[
        {
            "title":"Synthetic comment",
            "string_list_data":[
                {
                    "href":"https://example.invalid/p/synthetic-id",
                    "value":"Synthetic value",
                    "timestamp":123
                }
            ]
        }
    ]
}"#;

#[test]
fn recursive_activity_arrays_parse_for_each_supported_category() {
    for category in [
        ArchiveEntryCategory::SavedPosts,
        ArchiveEntryCategory::LikedPosts,
        ArchiveEntryCategory::StoriesViewed,
        ArchiveEntryCategory::AdWatchedVideos,
    ] {
        let summary =
            validate_json(category, ACTIVITY.as_bytes()).expect("synthetic activity parses");
        assert_eq!(
            summary,
            ParsedEntrySummary {
                record_count: 1,
                message_count: 0,
                reaction_count: 0,
            }
        );
    }
}

#[test]
fn observed_message_variants_and_missing_optional_fields_parse() {
    let summary = validate_json(ArchiveEntryCategory::MessageThread, THREAD.as_bytes())
        .expect("synthetic thread parses");
    assert_eq!(
        summary,
        ParsedEntrySummary {
            record_count: 0,
            message_count: 2,
            reaction_count: 2,
        }
    );
}

#[test]
fn thread_optional_root_fields_can_be_absent() {
    let minimal = br#"{
        "participants":[],
        "messages":[],
        "title":"",
        "is_still_participant":false,
        "thread_path":"",
        "magic_words":[]
    }"#;
    let summary = validate_json(ArchiveEntryCategory::MessageThread, minimal)
        .expect("optional root fields can be absent");
    assert_eq!(summary, ParsedEntrySummary::default());
}

#[test]
fn comment_likes_use_their_own_legacy_wrapper() {
    let summary = validate_json(ArchiveEntryCategory::LikedComments, COMMENTS.as_bytes())
        .expect("synthetic comment likes parse");
    assert_eq!(summary.record_count, 1);
    assert_eq!(summary.message_count, 0);
    assert_eq!(summary.reaction_count, 0);
    assert_eq!(
        validate_json(ArchiveEntryCategory::LikedPosts, COMMENTS.as_bytes()),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
}

#[test]
fn unknown_activity_fields_are_rejected_at_root_and_recursive_levels() {
    for (before, after) in [
        (
            r#""fbid":"synthetic-id""#,
            r#""unknown":"synthetic-private-value","fbid":"synthetic-id""#,
        ),
        (
            r#""label":"Synthetic leaf""#,
            r#""unknown":"synthetic-private-value","label":"Synthetic leaf""#,
        ),
        (
            r#""title":"Synthetic container""#,
            r#""unknown":"synthetic-private-value","title":"Synthetic container""#,
        ),
    ] {
        let invalid = ACTIVITY.replacen(before, after, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::SavedPosts, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}

#[test]
fn unknown_message_fields_are_rejected_at_every_observed_nested_level() {
    for (before, after) in [
        (
            r#""participants":"#,
            r#""unknown":"synthetic-private-value","participants":"#,
        ),
        (
            r#""name":"Synthetic sender""#,
            r#""unknown":"synthetic-private-value","name":"Synthetic sender""#,
        ),
        (
            r#""sender_name":"Synthetic sender""#,
            r#""unknown":"synthetic-private-value","sender_name":"Synthetic sender""#,
        ),
        (
            r#""share_text":"Synthetic share""#,
            r#""unknown":"synthetic-private-value","share_text":"Synthetic share""#,
        ),
        (
            r#""reaction":"heart""#,
            r#""unknown":"synthetic-private-value","reaction":"heart""#,
        ),
        (
            r#""uri":"synthetic/photo.jpg""#,
            r#""unknown":"synthetic-private-value","uri":"synthetic/photo.jpg""#,
        ),
        (
            r#""mode":1"#,
            r#""unknown":"synthetic-private-value","mode":1"#,
        ),
    ] {
        let invalid = THREAD.replacen(before, after, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::MessageThread, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}

#[test]
fn unknown_comment_fields_are_rejected_at_each_layer() {
    for (before, after) in [
        (
            r#""likes_comment_likes":"#,
            r#""unknown":"synthetic-private-value","likes_comment_likes":"#,
        ),
        (
            r#""title":"Synthetic comment""#,
            r#""unknown":"synthetic-private-value","title":"Synthetic comment""#,
        ),
        (
            r#""href":"https://example.invalid/p/synthetic-id""#,
            r#""unknown":"synthetic-private-value","href":"https://example.invalid/p/synthetic-id""#,
        ),
    ] {
        let invalid = COMMENTS.replacen(before, after, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::LikedComments, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}

#[test]
fn wrong_scalar_types_and_missing_required_fields_fail_sanitized() {
    for (before, after) in [
        (r#""timestamp":123"#, r#""timestamp":"123""#),
        (r#""timestamp":123"#, r#""timestamp":-1"#),
        (r#""timestamp":123"#, r#""timestamp":1.5"#),
        (r#""fbid":"synthetic-id","#, ""),
        (r#""media":[]"#, r#""media":null"#),
    ] {
        let invalid = ACTIVITY.replacen(before, after, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::SavedPosts, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
    let invalid = THREAD.replacen(
        r#""is_geoblocked_for_viewer":false"#,
        r#""is_geoblocked_for_viewer":"false""#,
        1,
    );
    assert_eq!(
        validate_json(ArchiveEntryCategory::MessageThread, invalid.as_bytes()),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
}

#[test]
fn arrays_without_observed_item_schema_fail_when_nonempty() {
    let invalid = ACTIVITY.replacen(r#""media":[]"#, r#""media":[{}]"#, 1);
    assert_eq!(
        validate_json(ArchiveEntryCategory::SavedPosts, invalid.as_bytes()),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
    for (before, after) in [
        (r#""media":[]"#, r#""media":[{}]"#),
        (r#""magic_words":[]"#, r#""magic_words":[{}]"#),
    ] {
        let invalid = THREAD.replacen(before, after, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::MessageThread, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}

#[test]
fn malformed_syntax_and_invalid_utf8_have_a_distinct_code() {
    for invalid in [b"[".as_slice(), b"[}".as_slice(), b"[@]".as_slice()] {
        assert_eq!(
            validate_json(ArchiveEntryCategory::SavedPosts, invalid),
            Err(ArchiveEntryErrorCode::InvalidJson)
        );
    }
    let mut invalid = ACTIVITY.as_bytes().to_vec();
    let index = invalid
        .iter()
        .position(|byte| *byte == b'S')
        .expect("fixture string");
    invalid[index] = 0xff;
    assert_eq!(
        validate_json(ArchiveEntryCategory::SavedPosts, &invalid),
        Err(ArchiveEntryErrorCode::InvalidJson)
    );
}

#[test]
fn classifier_requires_a_supported_section_path() {
    for (name, expected) in [
        (
            "your_instagram_activity/saved/saved_posts.json",
            ArchiveEntryCategory::SavedPosts,
        ),
        (
            "synthetic-root/your_instagram_activity/likes/liked_posts.json",
            ArchiveEntryCategory::LikedPosts,
        ),
        (
            "your_instagram_activity/likes/liked_comments.json",
            ArchiveEntryCategory::LikedComments,
        ),
        (
            "your_instagram_activity/story_interactions/stories_viewed.json",
            ArchiveEntryCategory::StoriesViewed,
        ),
        (
            "your_instagram_activity/messages/inbox/synthetic-thread/message_1.json",
            ArchiveEntryCategory::MessageThread,
        ),
        (
            "synthetic-root/your_instagram_activity/messages/message_requests/synthetic-thread/message_2.json",
            ArchiveEntryCategory::MessageThread,
        ),
    ] {
        assert_eq!(classify_entry(name), Some(expected));
    }
    for name in [
        "saved_posts.json",
        "unrelated/saved_posts.json",
        "your_instagram_activity/messages/inbox/synthetic-thread/message_.json",
        "your_instagram_activity/messages/inbox/synthetic-thread/message_a.json",
        "your_instagram_activity/messages/inbox/synthetic-thread/message_1.json/extra",
        "your_instagram_activity/messages/unknown/synthetic-thread/message_1.json",
    ] {
        assert_eq!(classify_entry(name), None);
    }
}

#[test]
fn required_scalars_cannot_be_null_or_coerced_from_other_json_types() {
    for (before, after) in [
        (r#""timestamp":123"#, r#""timestamp":null"#),
        (r#""timestamp":123"#, r#""timestamp":1e3"#),
        (r#""fbid":"synthetic-id""#, r#""fbid":null"#),
        (r#""fbid":"synthetic-id""#, r#""fbid":123"#),
        (r#""fbid":"synthetic-id""#, r#""fbid":true"#),
        (r#""media":[]"#, r#""media":null"#),
    ] {
        let invalid = ACTIVITY.replacen(before, after, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::SavedPosts, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
    for (before, after) in [
        (
            r#""is_still_participant":true"#,
            r#""is_still_participant":null"#,
        ),
        (
            r#""participants":[{"name":"Synthetic sender"},{"name":"Synthetic recipient"}]"#,
            r#""participants":null"#,
        ),
    ] {
        let invalid = THREAD.replacen(before, after, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::MessageThread, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}

#[test]
fn exactly_one_json_document_is_required_and_json_whitespace_is_allowed() {
    let summary = validate_json(ArchiveEntryCategory::SavedPosts, b" \r\n[] \t\n")
        .expect("standard JSON whitespace is accepted");
    assert_eq!(summary, ParsedEntrySummary::default());
    for invalid in [
        b"[] false".as_slice(),
        b"[] []".as_slice(),
        b"[]garbage".as_slice(),
    ] {
        assert_eq!(
            validate_json(ArchiveEntryCategory::SavedPosts, invalid),
            Err(ArchiveEntryErrorCode::InvalidJson)
        );
    }
}

#[test]
fn recursive_input_has_a_bounded_schema_depth() {
    let mut label = String::from(r#"{"label":"Synthetic leaf","value":"Synthetic value"}"#);
    for _ in 0..150 {
        label = format!(r#"{{"title":"Synthetic nesting","dict":[{label}]}}"#);
    }
    let invalid =
        format!(r#"[{{"fbid":"synthetic-id","timestamp":1,"media":[],"label_values":[{label}]}}]"#);
    assert_eq!(
        validate_json(ArchiveEntryCategory::SavedPosts, invalid.as_bytes()),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
}

#[test]
fn trailing_commas_and_leading_zero_numbers_are_invalid_json() {
    let invalid_array = ACTIVITY
        .trim_end()
        .strip_suffix(']')
        .expect("fixture array");
    let invalid_array = format!("{invalid_array},]");
    assert_eq!(
        validate_json(ArchiveEntryCategory::SavedPosts, invalid_array.as_bytes()),
        Err(ArchiveEntryErrorCode::InvalidJson)
    );
    let invalid_object = THREAD.trim_end().strip_suffix('}').expect("fixture object");
    let invalid_object = format!("{invalid_object},}}");
    assert_eq!(
        validate_json(
            ArchiveEntryCategory::MessageThread,
            invalid_object.as_bytes()
        ),
        Err(ArchiveEntryErrorCode::InvalidJson)
    );
    let invalid_number = ACTIVITY.replacen(r#""timestamp":123"#, r#""timestamp":01"#, 1);
    assert_eq!(
        validate_json(ArchiveEntryCategory::SavedPosts, invalid_number.as_bytes()),
        Err(ArchiveEntryErrorCode::InvalidJson)
    );
}

#[test]
fn duplicate_keys_cannot_replace_source_values() {
    let invalid = ACTIVITY.replacen(
        r#""timestamp":123"#,
        r#""timestamp":123,"timestamp":456"#,
        1,
    );
    assert_eq!(
        validate_json(ArchiveEntryCategory::SavedPosts, invalid.as_bytes()),
        Err(ArchiveEntryErrorCode::SchemaMismatch)
    );
}

#[test]
fn ad_watched_video_service_path_is_distinct_and_root_wrapping_is_supported() {
    for name in [
        "ads_information/ads_and_topics/videos_watched.json",
        "synthetic-root/ads_information/ads_and_topics/videos_watched.json",
    ] {
        assert_eq!(
            classify_entry(name),
            Some(ArchiveEntryCategory::AdWatchedVideos)
        );
    }
    assert_eq!(
        classify_entry("your_instagram_activity/ads_and_topics/videos_watched.json"),
        None
    );
}

#[test]
fn watched_video_empty_vectors_parse_but_nonempty_vectors_remain_unsupported() {
    let empty = ACTIVITY.replacen(
        r#""label":"Synthetic leaf""#,
        r#""vec":[],"label":"Synthetic leaf""#,
        1,
    );
    let summary = validate_json(ArchiveEntryCategory::AdWatchedVideos, empty.as_bytes())
        .expect("known empty vectors parse");
    assert_eq!(summary.record_count, 1);

    for vector in [r#""vec":[{}]"#, r#""vec":[{"unknown":"synthetic"}]"#] {
        let invalid = empty.replacen(r#""vec":[]"#, vector, 1);
        assert_eq!(
            validate_json(ArchiveEntryCategory::AdWatchedVideos, invalid.as_bytes()),
            Err(ArchiveEntryErrorCode::SchemaMismatch)
        );
    }
}
