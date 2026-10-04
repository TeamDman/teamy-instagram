use std::fs;
use std::fs::File;
use std::fs::FileTimes;
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use std::time::UNIX_EPOCH;
use teamy_cancellation::CancellationToken;
use teamy_instagram::catalog::CatalogError;
use teamy_instagram::catalog::DateBasis;
use teamy_instagram::catalog::ExportDate;
use teamy_instagram::catalog::InventoryLimits;
use teamy_instagram::catalog::RootsStore;
use teamy_instagram::catalog::inventory_roots;
use teamy_instagram::catalog::select_latest;
use tempfile::TempDir;

fn store(directory: &TempDir) -> RootsStore {
    RootsStore::new(directory.path().join("settings").join("archive-roots.json"))
}

fn archive(root: &Path, filename: &str, modified_seconds: u64) -> eyre::Result<()> {
    let file = File::create(root.join(filename))?;
    file.set_times(
        FileTimes::new().set_modified(UNIX_EPOCH + Duration::from_secs(modified_seconds)),
    )?;
    Ok(())
}

#[test]
fn canonical_roots_deduplicate_persist_and_remove_missing_stored_paths() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    let first = directory.path().join("first");
    let second = directory.path().join("second");
    fs::create_dir(&first)?;
    fs::create_dir(&second)?;
    let settings = store(&directory);
    assert_eq!(settings.list()?.roots, Vec::<String>::new());
    let initial = settings.add(&first)?;
    assert_eq!(initial.roots.len(), 1);
    assert_eq!(
        settings.add(&first.join(".")).expect("deduplicate"),
        initial
    );
    let both = settings.add(&second)?;
    assert_eq!(both.roots.len(), 2);
    assert_eq!(store(&directory).list()?, both);
    settings.remove(&first)?;
    let remaining = settings.list()?;
    assert_eq!(remaining.roots.len(), 1);
    fs::remove_dir(&second)?;
    assert_eq!(
        settings.remove(Path::new(&remaining.roots[0]))?.roots,
        Vec::<String>::new()
    );
    Ok(())
}

#[test]
fn malformed_existing_configuration_is_preserved_with_fixed_errors() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    let root = directory.path().join("root");
    fs::create_dir(&root)?;
    let settings_path = directory.path().join("settings.json");
    let settings = RootsStore::new(settings_path.clone());
    for contents in [
        r#"{"version":1,"roots":[],"PRIVATE_SYNTHETIC_FIELD":"PRIVATE_SYNTHETIC_VALUE"}"#,
        r#"{"version":"1","roots":[]}"#,
        r#"{"version":null,"roots":[]}"#,
        r#"{"version":2,"roots":[]}"#,
        r#"{"version":1,"roots":[]} trailing"#,
    ] {
        fs::write(&settings_path, contents)?;
        assert_eq!(
            settings.add(&root).err(),
            Some(CatalogError::ConfigurationInvalid)
        );
        assert_eq!(
            settings.remove(&root).err(),
            Some(CatalogError::ConfigurationInvalid)
        );
        assert_eq!(fs::read_to_string(&settings_path)?, contents);
        let diagnostic = settings.list().expect_err("invalid settings").to_string();
        assert!(!diagnostic.contains("PRIVATE_SYNTHETIC"));
        assert!(!diagnostic.contains(settings_path.to_str().expect("fixture UTF-8")));
    }
    Ok(())
}

#[test]
fn inventory_deduplicates_roots_and_requires_an_independent_date_basis() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    let root = directory.path().join("root");
    let nested = root.join("nested");
    fs::create_dir_all(&nested)?;
    archive(&root, "instagram-synthetic-2026-01-02-a.zip", 2000)?;
    archive(&nested, "instagram-synthetic-2026-01-03-b.ZiP", 1000)?;
    archive(&root, "instagram-synthetic-no-date.zip", 3000)?;
    archive(&root, "unrelated-2026-12-31.zip", 4000)?;
    let settings = store(&directory);
    settings.add(&root)?;
    let roots = settings.add(&nested)?;
    let report = inventory_roots(
        &roots,
        &InventoryLimits::default(),
        &CancellationToken::new(),
    )?;
    assert_eq!(report.archives.len(), 3);
    assert!(!report.is_incomplete());
    assert!(
        report
            .archives
            .iter()
            .all(|file| Path::new(&file.path).is_absolute())
    );
    let export_latest = select_latest(&report, DateBasis::ExportFilename);
    assert_eq!(export_latest.date_basis, DateBasis::ExportFilename);
    assert_eq!(export_latest.missing_date_count, 1);
    assert!(
        export_latest
            .archive
            .expect("dated export")
            .path
            .ends_with("b.ZiP")
    );
    let modified_latest = select_latest(&report, DateBasis::FileModified);
    assert_eq!(modified_latest.missing_date_count, 0);
    assert!(
        modified_latest
            .archive
            .expect("modified export")
            .path
            .ends_with("no-date.zip")
    );
    Ok(())
}

#[test]
fn unsupported_and_ambiguous_filename_dates_remain_missing() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    for name in [
        "instagram-synthetic-2026-02-30-a.zip",
        "instagram-synthetic-2026-01-01-2026-02-02.zip",
        "instagram-synthetic-no-date.zip",
    ] {
        archive(directory.path(), name, 1000)?;
    }
    let settings = store(&directory);
    let report = inventory_roots(
        &settings.add(directory.path())?,
        &InventoryLimits::default(),
        &CancellationToken::new(),
    )?;
    let latest = select_latest(&report, DateBasis::ExportFilename);
    assert_eq!(latest.candidate_count, 3);
    assert_eq!(latest.missing_date_count, 3);
    assert_eq!(latest.archive, None);
    assert!(!latest.inventory_incomplete);
    assert_eq!(
        ExportDate::parse("2026-02-30").err(),
        Some(CatalogError::InvalidExportDate)
    );
    Ok(())
}

#[test]
fn inventory_limits_and_missing_roots_report_partial_coverage() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    let root = directory.path().join("root");
    let nested = root.join("nested");
    fs::create_dir_all(&nested)?;
    for name in [
        "instagram-synthetic-2026-01-01-a.zip",
        "instagram-synthetic-2026-01-02-b.zip",
    ] {
        archive(&root, name, 1000)?;
    }
    archive(&nested, "instagram-synthetic-2026-01-03-c.zip", 1000)?;
    let settings = store(&directory);
    let roots = settings.add(&root)?;
    let bounded = inventory_roots(
        &roots,
        &InventoryLimits {
            max_entries: 2,
            ..InventoryLimits::default()
        },
        &CancellationToken::new(),
    )?;
    assert_eq!(bounded.scanned_entry_count, 2);
    assert!(bounded.entry_limit_reached);
    assert!(select_latest(&bounded, DateBasis::ExportFilename).inventory_incomplete);
    let shallow = inventory_roots(
        &roots,
        &InventoryLimits {
            recursive: false,
            ..InventoryLimits::default()
        },
        &CancellationToken::new(),
    )?;
    assert_eq!(shallow.archives.len(), 2);
    let missing = directory.path().join("missing");
    fs::create_dir(&missing)?;
    let roots = settings.add(&missing)?;
    fs::remove_dir(&missing)?;
    let partial = inventory_roots(
        &roots,
        &InventoryLimits::default(),
        &CancellationToken::new(),
    )?;
    assert_eq!(partial.unreadable_entry_count, 1);
    assert!(partial.is_incomplete());
    Ok(())
}

#[test]
fn latest_ties_are_deterministic_and_cancelled_inventory_reads_nothing() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    archive(
        directory.path(),
        "instagram-synthetic-2026-01-01-b.zip",
        1000,
    )?;
    archive(
        directory.path(),
        "instagram-synthetic-2026-01-01-a.zip",
        1000,
    )?;
    let roots = store(&directory).add(directory.path())?;
    let report = inventory_roots(
        &roots,
        &InventoryLimits::default(),
        &CancellationToken::new(),
    )?;
    assert!(
        select_latest(&report, DateBasis::ExportFilename)
            .archive
            .expect("tie winner")
            .path
            .ends_with("a.zip")
    );
    let cancellation = CancellationToken::new();
    cancellation.request_cancel("synthetic cancellation");
    assert_eq!(
        inventory_roots(&roots, &InventoryLimits::default(), &cancellation).err(),
        Some(CatalogError::Cancelled)
    );
    Ok(())
}

#[test]
fn catalog_cli_is_injectable_and_latest_requires_date_basis() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    let settings = directory.path().join("settings.json");
    let root = directory.path().join("root");
    fs::create_dir(&root)?;
    archive(&root, "instagram-synthetic-2026-01-01-a.zip", 1000)?;
    let add = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["roots", "--configuration-path"])
        .arg(&settings)
        .arg("add")
        .arg(&root)
        .output()?;
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let selected = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "latest", "--configuration-path"])
        .arg(&settings)
        .args(["--date-basis", "export-filename"])
        .output()?;
    assert!(
        selected.status.success(),
        "{}",
        String::from_utf8_lossy(&selected.stderr)
    );
    let selected_json = String::from_utf8(selected.stdout)?;
    assert!(selected_json.contains("\"date_basis\": \"export-filename\""));
    assert!(selected_json.contains("instagram-synthetic-2026-01-01-a.zip"));
    let missing_basis = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "latest", "--configuration-path"])
        .arg(&settings)
        .output()?;
    assert!(!missing_basis.status.success());
    Ok(())
}
