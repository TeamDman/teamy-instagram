use super::ArchiveRecord;
use super::CatalogError;
use super::ExportDate;
use super::InventoryLimits;
use super::InventoryReport;
use super::RootsConfiguration;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use teamy_cancellation::CancellationToken;

/// Inventory service-named ZIP files without opening their contents.
///
/// Traversal skips symlinks, uses entry/depth caps, and deduplicates overlapping
/// configured roots. Missing roots and unreadable entries are aggregate counts.
/// Filename dates do not imply a specific export time or a validated ZIP.
///
/// # Errors
/// Returns a fixed code for invalid limits or cancellation.
pub fn inventory_roots(
    configuration: &RootsConfiguration,
    limits: &InventoryLimits,
    cancellation: &CancellationToken,
) -> Result<InventoryReport, CatalogError> {
    if limits.max_entries == 0 || limits.max_depth == 0 {
        return Err(CatalogError::InvalidInventoryLimits);
    }
    cancellation
        .bail_if_cancelled()
        .map_err(|_error| CatalogError::Cancelled)?;
    let mut report = InventoryReport::default();
    let mut archives = BTreeMap::new();
    let mut directories: Vec<_> = configuration
        .roots
        .iter()
        .map(|root| (PathBuf::from(root), 0_usize))
        .collect();
    while let Some((directory, depth)) = directories.pop() {
        cancellation
            .bail_if_cancelled()
            .map_err(|_error| CatalogError::Cancelled)?;
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_error) => {
                report.unreadable_entry_count += 1;
                continue;
            }
        };
        for entry in entries {
            cancellation
                .bail_if_cancelled()
                .map_err(|_error| CatalogError::Cancelled)?;
            if report.scanned_entry_count >= u64::try_from(limits.max_entries).unwrap_or(u64::MAX) {
                report.entry_limit_reached = true;
                break;
            }
            report.scanned_entry_count += 1;
            let Ok(entry) = entry else {
                report.unreadable_entry_count += 1;
                continue;
            };
            let Ok(file_type) = entry.file_type() else {
                report.unreadable_entry_count += 1;
                continue;
            };
            if file_type.is_symlink() {
                report.skipped_symlink_count += 1;
                continue;
            }
            if file_type.is_dir() {
                if limits.recursive {
                    if depth < limits.max_depth {
                        directories.push((entry.path(), depth + 1));
                    } else {
                        report.depth_limited_directory_count += 1;
                    }
                }
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                continue;
            };
            let lowercase = name.to_ascii_lowercase();
            if !lowercase.starts_with("instagram-")
                || !std::path::Path::new(name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
            {
                continue;
            }
            let Ok(metadata) = entry.metadata() else {
                report.unreadable_entry_count += 1;
                continue;
            };
            let path = entry.path();
            let Some(path) = path.to_str() else {
                report.unreadable_entry_count += 1;
                continue;
            };
            let modified = metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .and_then(|duration| u64::try_from(duration.as_millis()).ok());
            archives
                .entry(path.to_owned())
                .or_insert_with(|| ArchiveRecord {
                    path: path.to_owned(),
                    size_bytes: metadata.len(),
                    modified_unix_milliseconds: modified,
                    export_date: ExportDate::from_filename(name),
                });
        }
        if report.entry_limit_reached {
            break;
        }
    }
    report.archives = archives.into_values().collect();
    Ok(report)
}
