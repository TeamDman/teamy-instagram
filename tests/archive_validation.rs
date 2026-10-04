use std::fs::File;
use std::io::Write;
use std::path::Path;
use teamy_cancellation::CancellationToken;
use teamy_instagram::archive::ArchiveEntryCategory;
use teamy_instagram::archive::ArchiveEntryErrorCode;
use teamy_instagram::archive::ArchiveEntryStatus;
use teamy_instagram::archive::ValidationLimits;
use teamy_instagram::archive::validate_archive;
use tempfile::TempDir;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

fn fixture(entries: &[(&str, &[u8])]) -> (TempDir, std::path::PathBuf) {
    let directory = TempDir::new().expect("fixture directory");
    let path = directory.path().join("synthetic.zip");
    let mut writer = ZipWriter::new(File::create(&path).expect("fixture file"));
    for (name, bytes) in entries {
        writer
            .start_file(
                *name,
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
            )
            .expect("fixture entry");
        writer.write_all(bytes).expect("fixture contents");
    }
    writer.finish().expect("finished fixture");
    (directory, path)
}

fn validate(path: &Path, limits: &ValidationLimits) -> teamy_instagram::archive::ValidationReport {
    validate_archive(path, limits, &CancellationToken::new()).expect("fixture validation")
}

#[test]
fn optional_sections_and_unsupported_entries_have_distinct_coverage() {
    let (_directory, path) = fixture(&[
        ("your_instagram_activity/saved/saved_posts.json", b"[]"),
        (
            "unrecognized/synthetic.json",
            b"this is deliberately not JSON",
        ),
        ("media/synthetic.mp4", b"this is deliberately not a video"),
        ("start_here.html", b"synthetic"),
    ]);
    let report = validate(&path, &ValidationLimits::default());
    assert_eq!(report.entry_count, 4);
    assert_eq!(report.parsed_count, 1);
    assert_eq!(report.parse_failure_count, 0);
    assert_eq!(report.unsupported_count, 2);
    assert_eq!(report.media_count, 1);
    assert_eq!(report.inspected_bytes, 2);
    assert!(!report.has_failures());
    assert!(!report.all_exposed_entries_parsed);
    assert_eq!(report.entries[0].category, ArchiveEntryCategory::SavedPosts);
    assert_eq!(report.entries[0].status, ArchiveEntryStatus::Parsed);
    assert_eq!(
        report.entries[0]
            .summary
            .as_ref()
            .expect("parsed summary")
            .record_count,
        0
    );
    assert_eq!(report.entries[1].status, ArchiveEntryStatus::Unsupported);
    assert_eq!(report.entries[2].status, ArchiveEntryStatus::Media);
}

#[test]
fn single_recognized_optional_section_can_validate() {
    let (_directory, path) = fixture(&[("your_instagram_activity/saved/saved_posts.json", b"[]")]);
    let report = validate(&path, &ValidationLimits::default());
    assert_eq!(report.parsed_count, 1);
    assert!(!report.has_failures());
    assert!(report.all_exposed_entries_parsed);
}

#[test]
fn parser_failures_do_not_leak_entry_names_or_values() {
    let (_directory, path) = fixture(&[(
        "your_instagram_activity/saved/saved_posts.json",
        b"[{\"synthetic_private_field\":\"synthetic_private_value\"}]",
    )]);
    let report = validate(&path, &ValidationLimits::default());
    assert_eq!(report.parse_failure_count, 1);
    assert!(report.has_failures());
    assert_eq!(
        report.entries[0].error_code,
        Some(ArchiveEntryErrorCode::SchemaMismatch)
    );
    assert!(report.entries[0].summary.is_none());
    let json = facet_json::to_string(&report).expect("report JSON");
    assert!(!json.contains("synthetic_private"));
    assert!(!json.contains("saved_posts.json"));
    assert!(!json.contains("your_instagram_activity"));
}

#[test]
fn malformed_known_json_is_a_failure() {
    let (_directory, path) = fixture(&[("your_instagram_activity/saved/saved_posts.json", b"[")]);
    let report = validate(&path, &ValidationLimits::default());
    assert_eq!(report.parse_failure_count, 1);
    assert_eq!(report.entries[0].status, ArchiveEntryStatus::ParseFailure);
}

#[test]
fn oversized_known_entry_is_reported_without_decompressing() {
    let (_directory, path) = fixture(&[(
        "your_instagram_activity/saved/saved_posts.json",
        b"[          ]",
    )]);
    let report = validate(
        &path,
        &ValidationLimits {
            max_entry_bytes: 4,
            ..ValidationLimits::default()
        },
    );
    assert_eq!(report.inspected_bytes, 0);
    assert_eq!(
        report.entries[0].error_code,
        Some(ArchiveEntryErrorCode::EntryTooLarge)
    );
}

#[test]
fn aggregate_budget_counts_present_recognized_entries() {
    let (_directory, path) = fixture(&[
        ("your_instagram_activity/saved/saved_posts.json", b"[]"),
        (
            "your_instagram_activity/likes/liked_comments.json",
            b"{\"likes_comment_likes\":[]}",
        ),
    ]);
    let report = validate(
        &path,
        &ValidationLimits {
            max_total_read_bytes: 2,
            ..ValidationLimits::default()
        },
    );
    assert_eq!(report.parsed_count, 1);
    assert_eq!(report.inspected_bytes, 2);
    assert_eq!(
        report.entries[1].error_code,
        Some(ArchiveEntryErrorCode::TotalReadLimit)
    );
}

#[test]
fn entry_count_cap_rejects_before_entry_reads() {
    let (_directory, path) = fixture(&[("one.txt", b""), ("two.txt", b"")]);
    let error = validate_archive(
        &path,
        &ValidationLimits {
            max_entries: 1,
            ..ValidationLimits::default()
        },
        &CancellationToken::new(),
    )
    .expect_err("entry cap should fail");
    assert_eq!(error.to_string(), "archive_entry_count_limit");
}

#[test]
fn cancellation_prevents_archive_access() {
    let cancellation = CancellationToken::new();
    cancellation.request_cancel("synthetic cancellation");
    let error = validate_archive(
        Path::new("synthetic-missing.zip"),
        &ValidationLimits::default(),
        &cancellation,
    )
    .expect_err("cancelled operation");
    assert_eq!(error.to_string(), "synthetic cancellation");
}

#[test]
fn invalid_archive_error_does_not_contain_its_path() {
    let directory = TempDir::new().expect("fixture directory");
    let path = directory.path().join("synthetic_private_archive.zip");
    std::fs::write(&path, b"not a ZIP").expect("invalid fixture");
    let error = validate_archive(
        &path,
        &ValidationLimits::default(),
        &CancellationToken::new(),
    )
    .expect_err("invalid ZIP should fail");
    assert!(error.to_string().starts_with("archive_structure_failed:"));
    assert!(!error.to_string().contains("synthetic_private_archive"));
}

#[test]
fn corrupt_known_entry_reports_read_failure_without_parser_details() {
    let (_directory, path) = fixture(&[("your_instagram_activity/saved/saved_posts.json", b"[]")]);
    let mut archive_bytes = std::fs::read(&path).expect("synthetic archive bytes");
    let directory_offset = archive_bytes
        .windows(4)
        .position(|window| window == [0x50, 0x4b, 0x01, 0x02])
        .expect("central directory marker");
    archive_bytes[directory_offset + 16..directory_offset + 20].fill(0);
    std::fs::write(&path, archive_bytes).expect("corrupt synthetic CRC");
    let report = validate(&path, &ValidationLimits::default());
    assert_eq!(report.parsed_count, 0);
    assert_eq!(report.parse_failure_count, 1);
    assert_eq!(
        report.entries[0].error_code,
        Some(ArchiveEntryErrorCode::EntryReadFailure)
    );
}

#[test]
fn invalid_limits_fail_before_accessing_a_missing_archive() {
    let limits = ValidationLimits {
        max_entry_bytes: 0,
        ..ValidationLimits::default()
    };
    let error = validate_archive(
        Path::new("synthetic-missing.zip"),
        &limits,
        &CancellationToken::new(),
    )
    .expect_err("zero entry bound");
    assert_eq!(error.to_string(), "archive_invalid_limits");
}

#[test]
fn unsupported_compression_does_not_open_unsupported_entries() {
    let (_directory, path) = fixture(&[
        ("unrecognized/synthetic.json", b"not JSON"),
        ("media/synthetic.mp4", b"not video"),
        ("your_instagram_activity/saved/saved_posts.json", b"[]"),
    ]);
    let mut bytes = std::fs::read(&path).expect("synthetic archive bytes");
    let directory_offsets: Vec<_> = bytes
        .windows(4)
        .enumerate()
        .filter_map(|(index, window)| (window == [0x50, 0x4b, 0x01, 0x02]).then_some(index))
        .collect();
    for offset in directory_offsets {
        bytes[offset + 10..offset + 12].fill(0xff);
    }
    std::fs::write(&path, bytes).expect("unsupported synthetic compression");
    let report = validate(&path, &ValidationLimits::default());
    assert_eq!(report.unsupported_count, 1);
    assert_eq!(report.media_count, 1);
    assert_eq!(report.parse_failure_count, 1);
    assert_eq!(report.inspected_bytes, 0);
    assert_eq!(report.entries[2].category, ArchiveEntryCategory::SavedPosts);
    assert_eq!(
        report.entries[2].error_code,
        Some(ArchiveEntryErrorCode::EntryUnavailable)
    );
}
