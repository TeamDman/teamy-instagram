use std::io::Write;
use std::process::Command;
use tempfile::TempDir;
use zip::CompressionMethod;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

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
fn cli_emits_report_then_fails_without_private_diagnostics() -> eyre::Result<()> {
    let (directory, path) = fixture(br#"[{"PRIVATE_SYNTHETIC_FIELD":"PRIVATE_SYNTHETIC_VALUE"}]"#)?;
    let log = directory.path().join("private-log.ndjson");
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["--debug", "--log-file"])
        .arg(&log)
        .args(["--output-format", "json", "archive", "validate"])
        .arg(&path)
        .output()?;
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("\"parse_failure_count\": 1"), "{stdout}");
    assert!(stdout.contains("SchemaMismatch"), "{stdout}");
    let stderr = String::from_utf8(output.stderr)?;
    let logs = std::fs::read_to_string(&log)?;
    for text in [&stdout, &stderr, &logs] {
        assert!(!text.contains("PRIVATE_SYNTHETIC"));
        assert!(!text.contains(path.to_str().expect("temporary path is UTF-8")));
        assert!(!text.contains("saved_posts.json"));
    }
    assert!(logs.contains("time.busy"));
    assert!(!stdout.contains("time.busy"));
    Ok(())
}

#[test]
fn cli_defaults_to_json_and_accepts_optional_absent_sections() -> eyre::Result<()> {
    let (_directory, path) = fixture(b"[]")?;
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "validate"])
        .arg(&path)
        .output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.starts_with('{'));
    assert!(stdout.contains("\"parsed_count\": 1"), "{stdout}");
    Ok(())
}
