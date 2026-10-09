use crate::activity_report::extract_archive;
use crate::archive::ValidationLimits;
use crate::cli::output::CliOutput;
use crate::identity::ExporterIdentity;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use std::path::Path;
use teamy_cancellation::CancellationToken;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct ActivityArgs {
    /// Local ZIP file to read. The path and file contents are not logged.
    #[facet(args::positional)]
    pub zip_path: String,
    /// Exact exported sender/actor label confirmed as you; repeat for aliases.
    #[facet(args::named, default)]
    pub self_label: Vec<String>,
    /// Emit aggregate counts and coverage without reel IDs, URLs, or events.
    #[facet(args::named, default)]
    pub summary: bool,
    /// Maximum uncompressed bytes per recognized JSON entry (default 32 MiB).
    #[facet(args::named)]
    pub max_entry_bytes: Option<u64>,
    /// Maximum total recognized JSON bytes read (default 512 MiB).
    #[facet(args::named)]
    pub max_total_bytes: Option<u64>,
}

impl ActivityArgs {
    /// # Errors
    /// Returns an error on unreadable ZIP, invalid identity mapping or limits,
    /// or cancellation. Recognized-entry failures are emitted in the report.
    pub fn invoke(self, cancellation: &CancellationToken) -> eyre::Result<CliOutput> {
        let mut limits = ValidationLimits::default();
        if let Some(value) = self.max_entry_bytes {
            limits.max_entry_bytes = value;
        }
        if let Some(value) = self.max_total_bytes {
            limits.max_total_read_bytes = value;
        }
        let identity = ExporterIdentity::new(self.self_label)?;
        // Setup, 100 ms per permitted MiB of JSON, and 1 ms per ZIP directory entry.
        let threshold = Some(
            std::time::Duration::from_secs(3)
                .saturating_add(std::time::Duration::from_millis(
                    limits
                        .max_total_read_bytes
                        .div_ceil(1024 * 1024)
                        .saturating_mul(100),
                ))
                .saturating_add(std::time::Duration::from_millis(
                    u64::try_from(limits.max_entries).unwrap_or(u64::MAX),
                )),
        );
        let report = extract_archive(Path::new(&self.zip_path), &limits, &identity, cancellation)?;
        let failed = report.has_failures();
        let output = if self.summary {
            CliOutput::facet(report.aggregate_summary(), threshold)
        } else {
            CliOutput::facet(report, threshold)
        };
        Ok(output.with_failure(failed))
    }
}
