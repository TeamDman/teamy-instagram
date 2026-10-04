use crate::archive::ArchiveEntryCategory;
use crate::archive::ArchiveEntryErrorCode;
use crate::archive::ArchiveEntryReport;
use crate::archive::ArchiveEntryStatus;
use crate::archive::ValidationLimits;
use crate::archive::ValidationReport;
use crate::models;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use teamy_cancellation::CancellationToken;
use zip::ZipArchive;
use zip::read::ZipFile;

/// Validate recognized JSON directly from a local ZIP without extracting entries.
///
/// Unsupported entries and media are inventoried without opening their bodies.
/// JSON reads are bounded, with a single extra byte used to detect a false size.
/// Schema diagnostics are reduced to fixed codes before they leave the library.
///
/// # Errors
///
/// Returns an error for cancellation, invalid limits, an unreadable archive,
/// malformed ZIP metadata, or an archive exceeding the entry-count cap.
/// Entry read, size, JSON, and schema failures are collected in the report.
pub fn validate_archive(
    path: &Path,
    limits: &ValidationLimits,
    cancellation: &CancellationToken,
) -> eyre::Result<ValidationReport> {
    visit_archive(
        path,
        limits,
        cancellation,
        &mut |category, _index, bytes| models::validate_json(category, bytes),
    )
}

pub(crate) fn visit_archive<F>(
    path: &Path,
    limits: &ValidationLimits,
    cancellation: &CancellationToken,
    visitor: &mut F,
) -> eyre::Result<ValidationReport>
where
    F: FnMut(
        ArchiveEntryCategory,
        usize,
        &[u8],
    ) -> Result<models::ParsedEntrySummary, ArchiveEntryErrorCode>,
{
    cancellation.bail_if_cancelled()?;
    validate_limits(limits)?;
    let file =
        File::open(path).map_err(|error| eyre::eyre!("archive_open_failed: {}", error.kind()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| eyre::eyre!("archive_structure_failed: {}", zip_error_category(&error)))?;
    if archive.len() > limits.max_entries {
        eyre::bail!("archive_entry_count_limit");
    }
    let mut report = ValidationReport::default();
    for index in 0..archive.len() {
        cancellation.bail_if_cancelled()?;
        let Some(name) = archive.name_for_index(index) else {
            report.push(failure(
                index,
                ArchiveEntryCategory::Unreadable,
                ArchiveEntryErrorCode::EntryUnavailable,
            ));
            continue;
        };
        if name.ends_with('/') {
            report.push(outcome(
                index,
                ArchiveEntryCategory::Directory,
                ArchiveEntryStatus::Directory,
            ));
            continue;
        }
        let Some(category) = models::classify_entry(name) else {
            let (category, status) = classify_unsupported(name);
            report.push(outcome(index, category, status));
            continue;
        };
        let Ok(mut entry) = archive.by_index(index) else {
            report.push(failure(
                index,
                category,
                ArchiveEntryErrorCode::EntryUnavailable,
            ));
            continue;
        };
        let remaining = limits
            .max_total_read_bytes
            .saturating_sub(report.inspected_bytes);
        let (entry_report, read_bytes) = validate_known_entry(
            &mut entry,
            index,
            category,
            limits,
            remaining,
            cancellation,
            visitor,
        )?;
        report.inspected_bytes = report.inspected_bytes.saturating_add(read_bytes);
        report.push(entry_report);
    }
    report.all_exposed_entries_parsed = report.parsed_count > 0
        && report.parse_failure_count == 0
        && report.unsupported_count == 0
        && report.media_count == 0;
    Ok(report)
}

fn validate_known_entry<F>(
    entry: &mut ZipFile<'_, File>,
    index: usize,
    category: ArchiveEntryCategory,
    limits: &ValidationLimits,
    remaining: u64,
    cancellation: &CancellationToken,
    visitor: &mut F,
) -> eyre::Result<(ArchiveEntryReport, u64)>
where
    F: FnMut(
        ArchiveEntryCategory,
        usize,
        &[u8],
    ) -> Result<models::ParsedEntrySummary, ArchiveEntryErrorCode>,
{
    if entry.size() > limits.max_entry_bytes {
        return Ok((
            failure(index, category, ArchiveEntryErrorCode::EntryTooLarge),
            0,
        ));
    }
    if remaining == 0 || entry.size() > remaining {
        return Ok((
            failure(index, category, ArchiveEntryErrorCode::TotalReadLimit),
            0,
        ));
    }
    let bound = limits.max_entry_bytes.min(remaining);
    let (bytes, read_error) = read_bounded(entry, bound, cancellation)?;
    let read_bytes = u64::try_from(bytes.len())?;
    let error = read_error.or((read_bytes > bound).then_some(
        if limits.max_entry_bytes <= remaining {
            ArchiveEntryErrorCode::EntryTooLarge
        } else {
            ArchiveEntryErrorCode::TotalReadLimit
        },
    ));
    if let Some(error) = error {
        return Ok((failure(index, category, error), read_bytes));
    }
    cancellation.bail_if_cancelled()?;
    let entry_report = match visitor(category, index, &bytes) {
        Ok(summary) => ArchiveEntryReport {
            index,
            category,
            status: ArchiveEntryStatus::Parsed,
            error_code: None,
            summary: Some(summary),
        },
        Err(error) => failure(index, category, error),
    };
    cancellation.bail_if_cancelled()?;
    Ok((entry_report, read_bytes))
}

fn validate_limits(limits: &ValidationLimits) -> eyre::Result<()> {
    if limits.max_entry_bytes == 0
        || limits.max_total_read_bytes == 0
        || limits.max_entries == 0
        || limits.max_entry_bytes == u64::MAX
    {
        eyre::bail!("archive_invalid_limits");
    }
    Ok(())
}

fn zip_error_category(error: &zip::result::ZipError) -> &'static str {
    match error {
        zip::result::ZipError::Io(_) => "io",
        zip::result::ZipError::FileNotFound => "entry_missing",
        _ => "invalid_or_unsupported_zip",
    }
}

fn read_bounded(
    entry: &mut impl Read,
    limit: u64,
    cancellation: &CancellationToken,
) -> eyre::Result<(Vec<u8>, Option<ArchiveEntryErrorCode>)> {
    let mut reader = entry.take(limit + 1);
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        cancellation.bail_if_cancelled()?;
        let count = match reader.read(&mut buffer) {
            Ok(0) => return Ok((bytes, None)),
            Ok(count) => count,
            Err(_error) => return Ok((bytes, Some(ArchiveEntryErrorCode::EntryReadFailure))),
        };
        bytes.extend_from_slice(&buffer[..count]);
    }
}

fn outcome(
    index: usize,
    category: ArchiveEntryCategory,
    status: ArchiveEntryStatus,
) -> ArchiveEntryReport {
    ArchiveEntryReport {
        index,
        category,
        status,
        error_code: None,
        summary: None,
    }
}

fn failure(
    index: usize,
    category: ArchiveEntryCategory,
    error_code: ArchiveEntryErrorCode,
) -> ArchiveEntryReport {
    ArchiveEntryReport {
        index,
        category,
        status: ArchiveEntryStatus::ParseFailure,
        error_code: Some(error_code),
        summary: None,
    }
}

fn classify_unsupported(name: &str) -> (ArchiveEntryCategory, ArchiveEntryStatus) {
    let extension = name.rsplit_once('.').map(|(_, extension)| extension);
    match extension {
        Some(extension) if extension.eq_ignore_ascii_case("json") => (
            ArchiveEntryCategory::UnsupportedJson,
            ArchiveEntryStatus::Unsupported,
        ),
        Some(extension)
            if [
                "jpg", "jpeg", "png", "gif", "webp", "heic", "mp4", "mov", "m4v", "mp3", "m4a",
                "aac", "ogg", "wav",
            ]
            .iter()
            .any(|media_extension| extension.eq_ignore_ascii_case(media_extension)) =>
        {
            (ArchiveEntryCategory::Media, ArchiveEntryStatus::Media)
        }
        _ => (ArchiveEntryCategory::Other, ArchiveEntryStatus::Unsupported),
    }
}
