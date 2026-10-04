use crate::archive::ValidationLimits;
use crate::archive::validate_archive;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use std::path::Path;
use teamy_cancellation::CancellationToken;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct ValidateArgs {
    /// Local ZIP file to read. The path and file contents are not logged.
    #[facet(args::positional)]
    pub zip_path: String,
    /// Maximum uncompressed bytes per recognized JSON entry (default 32 MiB).
    #[facet(args::named)]
    pub max_entry_bytes: Option<u64>,
    /// Maximum total recognized JSON bytes read (default 512 MiB).
    #[facet(args::named)]
    pub max_total_bytes: Option<u64>,
}

impl ValidateArgs {
    /// # Errors
    /// Returns an error on unreadable ZIP, invalid limits, or cancellation.
    pub fn invoke(self, cancellation: &CancellationToken) -> eyre::Result<CliOutput> {
        let mut limits = ValidationLimits::default();
        if let Some(value) = self.max_entry_bytes {
            limits.max_entry_bytes = value;
        }
        if let Some(value) = self.max_total_bytes {
            limits.max_total_read_bytes = value;
        }
        let report = validate_archive(Path::new(&self.zip_path), &limits, cancellation)?;
        let failed = report.has_failures();
        Ok(CliOutput::facet(report).with_failure(failed))
    }
}
