use std::time::Duration;
use teamy_cancellation::CancellationToken;
use teamy_instagram::catalog::RootsStore;
use teamy_instagram::cli::archive::inventory::InventoryArgs;
use teamy_instagram::cli::archive::validate::ValidateArgs;
use teamy_instagram::cli::roots::list::RootsListArgs;

#[test]
fn representative_commands_return_work_based_budgets() -> eyre::Result<()> {
    let directory = tempfile::TempDir::new()?;
    let settings = directory.path().join("synthetic-settings.json");
    let store = RootsStore::new(settings.clone());
    assert_eq!(
        RootsListArgs {}
            .invoke(&store)?
            .elapsed_duration_warn_threshold,
        Some(Duration::from_secs(1))
    );
    for maximum in [10, 20] {
        let output = InventoryArgs {
            configuration_path: Some(settings.to_string_lossy().into_owned()),
            shallow: true,
            max_entries: Some(maximum),
        }
        .invoke(&CancellationToken::new())?;
        assert_eq!(
            output.elapsed_duration_warn_threshold,
            Some(Duration::from_millis(2000 + maximum * 2))
        );
    }
    let zip_path = directory.path().join("synthetic.zip");
    zip::ZipWriter::new(std::fs::File::create(&zip_path)?).finish()?;
    for maximum in [1024 * 1024, 2 * 1024 * 1024] {
        let output = ValidateArgs {
            zip_path: zip_path.to_string_lossy().into_owned(),
            max_entry_bytes: None,
            max_total_bytes: Some(maximum),
        }
        .invoke(&CancellationToken::new())?;
        assert_eq!(
            output.elapsed_duration_warn_threshold,
            Some(Duration::from_millis(
                103_000 + maximum.div_ceil(1024 * 1024) * 100
            ))
        );
    }
    Ok(())
}
