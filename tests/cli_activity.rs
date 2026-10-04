use std::io::Write;
use std::process::Command;
use teamy_instagram::cli::Cli;
use teamy_instagram::cli::Command as CliCommand;
use teamy_instagram::cli::archive::ArchiveCommand;
use tempfile::TempDir;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

const SYNTHETIC_REEL: &str = "SyntheticReel01";
const SYNTHETIC_URL: &str = "https://www.instagram.com/reel/SyntheticReel01/";
const SYNTHETIC_SELF: &str = "Synthetic self label";

fn fixture(bytes: &[u8]) -> eyre::Result<(TempDir, std::path::PathBuf)> {
    let directory = TempDir::new()?;
    let path = directory.path().join("synthetic-export.zip");
    let mut archive = ZipWriter::new(std::fs::File::create(&path)?);
    archive.start_file(
        "your_instagram_activity/saved/saved_posts.json",
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated),
    )?;
    archive.write_all(bytes)?;
    archive.finish()?;
    Ok((directory, path))
}

#[test]
fn activity_cli_parses_repeated_explicit_self_labels_and_limits() {
    let cli: Cli = figue::from_slice(&[
        "archive",
        "activity",
        "synthetic-export.zip",
        "--self-label",
        SYNTHETIC_SELF,
        "--self-label",
        "Synthetic alternate label",
        "--summary",
        "--max-entry-bytes",
        "4096",
        "--max-total-bytes",
        "8192",
    ])
    .unwrap();
    assert_eq!(cli.command.name(), "archive activity");
    let CliCommand::Archive(archive) = cli.command else {
        panic!("expected archive command");
    };
    let ArchiveCommand::Activity(activity) = archive.command else {
        panic!("expected activity command");
    };
    assert_eq!(activity.zip_path, "synthetic-export.zip");
    assert_eq!(
        activity.self_label,
        [SYNTHETIC_SELF, "Synthetic alternate label"]
    );
    assert!(activity.summary);
    assert_eq!(activity.max_entry_bytes, Some(4096));
    assert_eq!(activity.max_total_bytes, Some(8192));
}

#[test]
fn activity_cli_defaults_to_unresolved_identity_and_detailed_output() {
    let cli: Cli = figue::from_slice(&["archive", "activity", "synthetic-export.zip"]).unwrap();
    let CliCommand::Archive(archive) = cli.command else {
        panic!("expected archive command");
    };
    let ArchiveCommand::Activity(activity) = archive.command else {
        panic!("expected activity command");
    };
    assert_eq!(activity.self_label, Vec::<String>::new());
    assert!(!activity.summary);
}

#[test]
fn activity_cli_summary_excludes_reel_values_identity_and_paths() -> eyre::Result<()> {
    let record = format!(
        r#"[{{"fbid":"synthetic-record","timestamp":123,"media":[],"label_values":[{{"href":"{SYNTHETIC_URL}"}}]}}]"#,
    );
    let (directory, path) = fixture(record.as_bytes())?;
    let log = directory.path().join("synthetic-log.ndjson");
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["--debug", "--log-file"])
        .arg(&log)
        .args(["archive", "activity"])
        .arg(&path)
        .args(["--summary", "--self-label", SYNTHETIC_SELF])
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with('{'));
    assert!(stdout.contains("\"unique_reel_count\": 1"), "{stdout}");
    assert!(
        stdout.contains("\"activity_occurrence_count\": 1"),
        "{stdout}"
    );
    assert!(stdout.contains("Bookmarked"), "{stdout}");
    let stderr = String::from_utf8(output.stderr)?;
    let logs = std::fs::read_to_string(&log)?;
    for text in [&stdout, &stderr, &logs] {
        assert!(!text.contains(SYNTHETIC_REEL));
        assert!(!text.contains(SYNTHETIC_URL));
        assert!(!text.contains(SYNTHETIC_SELF));
        assert!(!text.contains("synthetic-record"));
        assert!(!text.contains(path.to_str().expect("temporary path is UTF-8")));
        assert!(!text.contains("saved_posts.json"));
    }
    assert!(logs.contains("archive activity"), "{logs}");
    Ok(())
}

#[test]
fn activity_cli_detailed_output_lists_normalized_reel_reference() -> eyre::Result<()> {
    let record = format!(
        r#"[{{"fbid":"synthetic-record","timestamp":123,"media":[],"label_values":[{{"href":"{SYNTHETIC_URL}?synthetic=tracking"}}]}}]"#,
    );
    let (_directory, path) = fixture(record.as_bytes())?;
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "activity"])
        .arg(&path)
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains(SYNTHETIC_URL), "{stdout}");
    assert!(stdout.contains("Bookmarked"), "{stdout}");
    assert!(!stdout.contains("synthetic=tracking"), "{stdout}");
    assert!(!stdout.contains("synthetic-record"), "{stdout}");
    Ok(())
}

#[test]
fn activity_cli_emits_report_then_fails_with_safe_schema_error() -> eyre::Result<()> {
    let (_directory, path) =
        fixture(br#"[{"PRIVATE_SYNTHETIC_FIELD":"PRIVATE_SYNTHETIC_VALUE"}]"#)?;
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "activity"])
        .arg(&path)
        .arg("--summary")
        .output()?;
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with('{'));
    assert!(stdout.contains("SchemaMismatch"), "{stdout}");
    let stderr = String::from_utf8(output.stderr)?;
    for text in [&stdout, &stderr] {
        assert!(!text.contains("PRIVATE_SYNTHETIC"));
        assert!(!text.contains(path.to_str().expect("temporary path is UTF-8")));
        assert!(!text.contains("saved_posts.json"));
    }
    Ok(())
}
