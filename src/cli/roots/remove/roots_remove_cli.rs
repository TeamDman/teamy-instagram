use crate::catalog::RootsStore;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use std::path::Path;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RootsRemoveArgs {
    /// Root directory or exact stored canonical path to remove.
    #[facet(args::positional)]
    pub path: String,
}

impl RootsRemoveArgs {
    /// # Errors
    /// Returns a fixed code for a non-UTF-8 path or settings failure.
    pub fn invoke(self, store: &RootsStore) -> eyre::Result<CliOutput> {
        // A small local settings read/write and path check.
        Ok(CliOutput::facet(
            store.remove(Path::new(&self.path))?,
            Some(std::time::Duration::from_secs(1)),
        ))
    }
}
