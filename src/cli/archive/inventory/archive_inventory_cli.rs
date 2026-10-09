use crate::catalog::InventoryLimits;
use crate::catalog::RootsStore;
use crate::catalog::inventory_roots;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use std::path::PathBuf;
use teamy_cancellation::CancellationToken;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct InventoryArgs {
    /// Optional settings file; otherwise use application local appdata.
    #[facet(args::named)]
    pub configuration_path: Option<String>,
    /// Scan only directly within each configured root.
    #[facet(args::named, default)]
    pub shallow: bool,
    /// Maximum filesystem entries visited (default 100,000).
    #[facet(args::named)]
    pub max_entries: Option<u64>,
}

impl InventoryArgs {
    /// # Errors
    /// Returns fixed settings/limit errors or cancellation. Incomplete
    /// filesystem coverage is explicit in the report.
    pub fn invoke(self, cancellation: &CancellationToken) -> eyre::Result<CliOutput> {
        let store = match self.configuration_path {
            Some(path) => RootsStore::new(PathBuf::from(path)),
            None => RootsStore::in_appdata()?,
        };
        let mut limits = InventoryLimits {
            recursive: !self.shallow,
            ..InventoryLimits::default()
        };
        if let Some(maximum) = self.max_entries {
            limits.max_entries = maximum
                .try_into()
                .map_err(|_error| crate::catalog::CatalogError::InvalidInventoryLimits)?;
        }
        // Directory traversal and rendering: setup plus 2 ms per permitted entry.
        let threshold = Some(
            std::time::Duration::from_secs(2).saturating_add(std::time::Duration::from_millis(
                u64::try_from(limits.max_entries)
                    .unwrap_or(u64::MAX)
                    .saturating_mul(2),
            )),
        );
        Ok(CliOutput::facet(
            inventory_roots(&store.list()?, &limits, cancellation)?,
            threshold,
        ))
    }
}
