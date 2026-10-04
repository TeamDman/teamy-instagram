use std::process::Command;
use teamy_instagram::catalog::DateBasis;
use teamy_instagram::catalog::InventoryReport;
use teamy_instagram::catalog::LatestReport;
use teamy_instagram::catalog::RootsConfiguration;
use teamy_instagram::cli::Cli;
use teamy_instagram::cli::Command as CliCommand;
use teamy_instagram::cli::archive::ArchiveCommand;
use teamy_instagram::cli::roots::RootsCommand;
use tempfile::TempDir;

#[test]
fn roots_cli_parses_configuration_override_and_add_path() {
    let cli: Cli = figue::from_slice(&[
        "roots",
        "--configuration-path",
        "synthetic-settings.json",
        "add",
        "synthetic-root",
    ])
    .unwrap();
    assert_eq!(cli.command.name(), "roots add");
    let CliCommand::Roots(roots) = cli.command else {
        panic!("expected roots command");
    };
    assert_eq!(
        roots.configuration_path.as_deref(),
        Some("synthetic-settings.json")
    );
    let RootsCommand::Add(args) = roots.command else {
        panic!("expected roots add command");
    };
    assert_eq!(args.path, "synthetic-root");
}

#[test]
fn latest_cli_parses_explicit_date_basis_and_inventory_limits() {
    let cli: Cli = figue::from_slice(&[
        "archive",
        "latest",
        "--date-basis",
        "export-filename",
        "--configuration-path",
        "synthetic-settings.json",
        "--shallow",
        "--max-entries",
        "10",
    ])
    .unwrap();
    assert_eq!(cli.command.name(), "archive latest");
    let CliCommand::Archive(archive) = cli.command else {
        panic!("expected archive command");
    };
    let ArchiveCommand::Latest(args) = archive.command else {
        panic!("expected latest command");
    };
    assert_eq!(args.date_basis, Some(DateBasis::ExportFilename));
    assert_eq!(
        args.configuration_path.as_deref(),
        Some("synthetic-settings.json")
    );
    assert!(args.shallow);
    assert_eq!(args.max_entries, Some(10));
}

#[test]
fn latest_cli_requires_an_explicit_date_basis() -> eyre::Result<()> {
    let output = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "latest"])
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)?.contains("catalog_date_basis_required"));
    Ok(())
}

#[test]
fn catalog_cli_registers_inventories_selects_and_removes_synthetic_root() -> eyre::Result<()> {
    let directory = TempDir::new()?;
    let root = directory.path().join("synthetic-exports");
    std::fs::create_dir(&root)?;
    let settings = directory.path().join("synthetic-settings.json");
    let newest = root.join("instagram-synthetic-2001-02-03-token.zip");
    let older = root.join("instagram-synthetic-2000-01-02-token.zip");
    let ignored = root.join("unrelated.zip");
    for path in [&newest, &older, &ignored] {
        zip::ZipWriter::new(std::fs::File::create(path)?).finish()?;
    }

    let added = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["roots", "--configuration-path"])
        .arg(&settings)
        .arg("add")
        .arg(&root)
        .output()?;
    assert!(added.status.success());
    let roots: RootsConfiguration = facet_json::from_slice(&added.stdout)?;
    assert_eq!(roots.roots.len(), 1);
    assert_eq!(std::path::Path::new(&roots.roots[0]), root.canonicalize()?);

    let listed = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["roots", "--configuration-path"])
        .arg(&settings)
        .arg("list")
        .output()?;
    assert!(listed.status.success());
    assert_eq!(
        roots,
        facet_json::from_slice::<RootsConfiguration>(&listed.stdout)?
    );

    let inventoried = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "inventory", "--configuration-path"])
        .arg(&settings)
        .output()?;
    assert!(inventoried.status.success());
    let inventory: InventoryReport = facet_json::from_slice(&inventoried.stdout)?;
    assert_eq!(inventory.archives.len(), 2);
    assert!(!inventory.is_incomplete());

    let selected = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["archive", "latest", "--configuration-path"])
        .arg(&settings)
        .args(["--date-basis", "export-filename"])
        .output()?;
    assert!(selected.status.success());
    let latest: LatestReport = facet_json::from_slice(&selected.stdout)?;
    assert_eq!(latest.date_basis, DateBasis::ExportFilename);
    assert_eq!(latest.candidate_count, 2);
    assert_eq!(latest.missing_date_count, 0);
    let archive = latest
        .archive
        .expect("synthetic archives have export dates");
    assert_eq!(std::path::Path::new(&archive.path), newest.canonicalize()?);

    let removed = Command::new(env!("CARGO_BIN_EXE_teamy-instagram"))
        .args(["roots", "--configuration-path"])
        .arg(&settings)
        .arg("remove")
        .arg(&root)
        .output()?;
    assert!(removed.status.success());
    assert_eq!(
        facet_json::from_slice::<RootsConfiguration>(&removed.stdout)?
            .roots
            .len(),
        0
    );
    Ok(())
}
