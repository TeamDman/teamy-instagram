use super::ArchiveRecord;
use super::DateBasis;
use super::InventoryReport;
use facet::Facet;

/// Missing dates are counted and skipped, with no fallback to another basis.
#[derive(Facet, Debug, Clone, PartialEq, Eq)]
pub struct LatestReport {
    pub date_basis: DateBasis,
    pub archive: Option<ArchiveRecord>,
    pub candidate_count: u64,
    pub missing_date_count: u64,
    pub inventory_incomplete: bool,
}

/// Select using only the chosen date source. Equal dates choose the
/// lexicographically smallest canonical path, independent of traversal order.
#[must_use]
pub fn select_latest(inventory: &InventoryReport, basis: DateBasis) -> LatestReport {
    let mut selected: Option<&ArchiveRecord> = None;
    let mut missing_date_count = 0;
    for archive in &inventory.archives {
        let has_date = match basis {
            DateBasis::ExportFilename => archive.export_date.is_some(),
            DateBasis::FileModified => archive.modified_unix_milliseconds.is_some(),
        };
        if !has_date {
            missing_date_count += 1;
            continue;
        }
        let replace = selected.is_none_or(|current| {
            let ordering = match basis {
                DateBasis::ExportFilename => archive.export_date.cmp(&current.export_date),
                DateBasis::FileModified => archive
                    .modified_unix_milliseconds
                    .cmp(&current.modified_unix_milliseconds),
            };
            ordering.is_gt() || (ordering.is_eq() && archive.path < current.path)
        });
        if replace {
            selected = Some(archive);
        }
    }
    LatestReport {
        date_basis: basis,
        archive: selected.cloned(),
        candidate_count: u64::try_from(inventory.archives.len()).unwrap_or(u64::MAX),
        missing_date_count,
        inventory_incomplete: inventory.is_incomplete(),
    }
}
