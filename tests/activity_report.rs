use std::fs::File;
use std::io::Write;
use teamy_cancellation::CancellationToken;
use teamy_instagram::activity::ActivityKind;
use teamy_instagram::activity_report::ActivityCoverageStatus;
use teamy_instagram::activity_report::ReelActivityReport;
use teamy_instagram::activity_report::extract_archive;
use teamy_instagram::archive::ValidationLimits;
use teamy_instagram::identity::ExporterIdentity;
use tempfile::TempDir;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const RECORD: &[u8] = br#"[{"fbid":"synthetic","timestamp":123,"media":[],"label_values":[{"href":"https://instagram.com/reel/SyntheticReel/","value":"https://instagram.com/reel/SyntheticReel/?tracking=synthetic"}]}]"#;
const THREAD: &[u8] = br#"{"participants":[{"name":"Synthetic Self"},{"name":"Synthetic Friend"}],"messages":[{"sender_name":"Synthetic Self","timestamp_ms":123000,"is_geoblocked_for_viewer":false,"share":{"link":"https://instagram.com/reel/SyntheticReel/"}},{"sender_name":"Synthetic Friend","timestamp_ms":124000,"is_geoblocked_for_viewer":false,"share":{"link":"https://instagram.com/reel/SyntheticReel/?tracking=synthetic"},"reactions":[{"reaction":"heart","actor":"Synthetic Self","timestamp":125},{"reaction":"other","actor":"Synthetic Friend"}]}],"title":"Synthetic Thread","is_still_participant":true,"thread_path":"synthetic/thread","magic_words":[]}"#;

fn extract(
    entries: &[(&str, &[u8])],
    identity: &ExporterIdentity,
) -> eyre::Result<ReelActivityReport> {
    let directory = TempDir::new()?;
    let path = directory.path().join("synthetic.zip");
    let mut writer = ZipWriter::new(File::create(&path)?);
    for (name, bytes) in entries {
        writer.start_file(*name, SimpleFileOptions::default())?;
        writer.write_all(bytes)?;
    }
    writer.finish()?;
    extract_archive(
        &path,
        &ValidationLimits::default(),
        identity,
        &CancellationToken::new(),
    )
}

fn status(report: &ReelActivityReport, kind: ActivityKind) -> ActivityCoverageStatus {
    report
        .summary
        .coverage
        .iter()
        .find(|coverage| coverage.activity == kind)
        .expect("six activity coverage entries")
        .status
}

#[test]
fn all_six_activities_group_one_identity_without_losing_occurrences() -> eyre::Result<()> {
    let identity = ExporterIdentity::new(vec!["Synthetic Self".to_owned()])?;
    let report = extract(
        &[
            ("your_instagram_activity/saved/saved_posts.json", RECORD),
            ("your_instagram_activity/likes/liked_posts.json", RECORD),
            (
                "your_instagram_activity/story_interactions/stories_viewed.json",
                RECORD,
            ),
            ("ads_information/ads_and_topics/videos_watched.json", RECORD),
            (
                "your_instagram_activity/messages/inbox/synthetic/message_1.json",
                THREAD,
            ),
        ],
        &identity,
    )?;
    assert!(!report.has_failures());
    assert_eq!(report.summary.unique_reel_count, 1);
    assert_eq!(report.summary.activity_occurrence_count, 11);
    assert_eq!(report.reels[0].evidence_indices.len(), 11);
    assert_eq!(report.summary.excluded_other_reactions, 1);
    assert_eq!(
        report.summary.reference_counts.recognized_reel_occurrences,
        10
    );
    for coverage in &report.summary.coverage {
        assert_eq!(coverage.status, ActivityCoverageStatus::Present);
        assert_eq!(coverage.distinct_reel_count, 1);
        assert_eq!(
            coverage.source_event_count,
            if coverage.activity == ActivityKind::Seen {
                2
            } else {
                1
            }
        );
    }
    assert!(!report.summary.complete_watch_history);
    let json = facet_json::to_string(&report)?;
    assert!(!json.contains("Synthetic Self"));
    assert!(!json.contains("Synthetic Friend"));
    assert!(!json.contains("tracking=synthetic"));
    assert!(json.contains("SyntheticReel"));
    let aggregate = facet_json::to_string(&report.aggregate_summary())?;
    assert!(!aggregate.contains("SyntheticReel"));
    assert!(!aggregate.contains("instagram.com"));
    Ok(())
}

#[test]
fn unresolved_direction_retains_shared_and_reaction_evidence() -> eyre::Result<()> {
    let report = extract(
        &[(
            "your_instagram_activity/messages/inbox/synthetic/message_1.json",
            THREAD,
        )],
        &ExporterIdentity::default(),
    )?;
    for kind in [
        ActivityKind::Sent,
        ActivityKind::Received,
        ActivityKind::ReactedTo,
    ] {
        assert_eq!(status(&report, kind), ActivityCoverageStatus::Ambiguous);
    }
    assert_eq!(report.summary.unresolved_shared_messages, 2);
    assert_eq!(report.summary.unresolved_reactions, 2);
    assert_eq!(report.activities.len(), 4);
    assert_eq!(report.reels.len(), 1);
    assert!(!report.has_failures());
    Ok(())
}

#[test]
fn missing_optional_sources_are_absent_without_parse_failure() -> eyre::Result<()> {
    let report = extract(
        &[("synthetic/unsupported.json", b"unread")],
        &ExporterIdentity::default(),
    )?;
    assert!(!report.has_failures());
    assert_eq!(report.summary.validation.unsupported_count, 1);
    assert_eq!(report.summary.coverage.len(), 6);
    for coverage in &report.summary.coverage {
        assert_eq!(coverage.status, ActivityCoverageStatus::Absent);
        assert_eq!(coverage.source_entry_count, 0);
    }
    Ok(())
}

#[test]
fn strict_failure_is_distinct_from_ambiguous_and_absent_activities() -> eyre::Result<()> {
    let report = extract(
        &[(
            "your_instagram_activity/likes/liked_posts.json",
            br#"[{"synthetic_unknown":true}]"#,
        )],
        &ExporterIdentity::default(),
    )?;
    assert!(report.has_failures());
    assert_eq!(
        status(&report, ActivityKind::Liked),
        ActivityCoverageStatus::Unsupported
    );
    assert_eq!(
        status(&report, ActivityKind::Seen),
        ActivityCoverageStatus::Absent
    );
    assert_eq!(
        report
            .summary
            .coverage
            .iter()
            .find(|item| item.activity == ActivityKind::Liked)
            .expect("liked coverage")
            .failed_source_entry_count,
        1
    );
    Ok(())
}

#[test]
fn opaque_reel_shares_are_ambiguous_while_posts_are_not_reel_evidence() -> eyre::Result<()> {
    let opaque = br#"[{"fbid":"synthetic","timestamp":1,"media":[],"label_values":[{"href":"https://instagram.com/share/reel/syntheticToken/"}]}]"#;
    let report = extract(
        &[("your_instagram_activity/saved/saved_posts.json", opaque)],
        &ExporterIdentity::default(),
    )?;
    assert_eq!(
        status(&report, ActivityKind::Bookmarked),
        ActivityCoverageStatus::Ambiguous
    );
    assert_eq!(
        report.summary.reference_counts.ambiguous_reel_occurrences,
        1
    );
    assert_eq!(report.reels.len(), 0);
    let posts = br#"[{"fbid":"synthetic","timestamp":1,"media":[],"label_values":[{"href":"https://instagram.com/p/SyntheticPost/"}]}]"#;
    let report = extract(
        &[("your_instagram_activity/saved/saved_posts.json", posts)],
        &ExporterIdentity::default(),
    )?;
    assert_eq!(
        status(&report, ActivityKind::Bookmarked),
        ActivityCoverageStatus::Absent
    );
    assert_eq!(
        report
            .summary
            .reference_counts
            .unsupported_instagram_occurrences,
        1
    );
    assert!(!report.has_failures());
    Ok(())
}

#[test]
fn opaque_share_ambiguity_follows_known_direction_and_reaction_presence() -> eyre::Result<()> {
    let identity = ExporterIdentity::new(vec!["Synthetic Self".to_owned()])?;
    let data = String::from_utf8(THREAD.to_vec())?.replace(
        "https://instagram.com/reel/SyntheticReel/",
        "https://instagram.com/share/reel/syntheticToken/",
    );
    let report = extract(
        &[(
            "your_instagram_activity/messages/inbox/synthetic/message_1.json",
            data.as_bytes(),
        )],
        &identity,
    )?;
    assert_eq!(
        status(&report, ActivityKind::Sent),
        ActivityCoverageStatus::Ambiguous
    );
    assert_eq!(
        status(&report, ActivityKind::Received),
        ActivityCoverageStatus::Ambiguous
    );
    assert_eq!(
        status(&report, ActivityKind::ReactedTo),
        ActivityCoverageStatus::Ambiguous
    );
    let without_reactions = br#"{"participants":[{"name":"Synthetic Self"},{"name":"Synthetic Friend"}],"messages":[{"sender_name":"Synthetic Self","timestamp_ms":123000,"is_geoblocked_for_viewer":false,"share":{"link":"https://instagram.com/share/reel/syntheticToken/"}}],"title":"Synthetic Thread","is_still_participant":true,"thread_path":"synthetic/thread","magic_words":[]}"#;
    let report = extract(
        &[(
            "your_instagram_activity/messages/inbox/synthetic/message_1.json",
            without_reactions,
        )],
        &identity,
    )?;
    assert_eq!(
        status(&report, ActivityKind::Sent),
        ActivityCoverageStatus::Ambiguous
    );
    assert_eq!(
        status(&report, ActivityKind::Received),
        ActivityCoverageStatus::Absent
    );
    assert_eq!(
        status(&report, ActivityKind::ReactedTo),
        ActivityCoverageStatus::Absent
    );
    Ok(())
}

#[test]
fn caption_references_remain_available_without_inflating_liked_reels() -> eyre::Result<()> {
    let data=br#"[{"fbid":"synthetic","timestamp":1,"media":[],"label_values":[{"label":"URL","href":"https://instagram.com/reel/SyntheticLiked/","value":"https://instagram.com/reel/SyntheticLiked/"},{"label":"Caption","value":"Synthetic caption mentions https://instagram.com/reel/SyntheticMention/"}]}]"#;
    let report = extract(
        &[("your_instagram_activity/likes/liked_posts.json", data)],
        &ExporterIdentity::default(),
    )?;
    let liked = report
        .summary
        .coverage
        .iter()
        .find(|item| item.activity == ActivityKind::Liked)
        .expect("liked coverage");
    assert_eq!(liked.distinct_reel_count, 1);
    assert_eq!(liked.source_event_count, 1);
    assert_eq!(liked.occurrence_count, 2);
    assert_eq!(report.summary.unique_reel_count, 1);
    assert_eq!(report.summary.incidental_reference_count, 1);
    assert_eq!(report.reels.len(), 1);
    assert_eq!(report.incidental_references.len(), 1);
    assert_eq!(
        report.incidental_references[0].reel.reel_id.as_str(),
        "SyntheticMention"
    );
    let aggregate = facet_json::to_string(&report.aggregate_summary())?;
    assert!(!aggregate.contains("SyntheticMention"));
    Ok(())
}
