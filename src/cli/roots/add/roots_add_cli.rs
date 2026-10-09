use crate::catalog::RootsStore;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use std::path::Path;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct RootsAddArgs {
    /// Existing directory to register. Paths are output only, never logged.
    #[facet(args::positional)]
    pub path: String,
}

impl RootsAddArgs {
    /// # Errors
    /// Returns a fixed code for an invalid directory or settings failure.
    pub fn invoke(self, store: &RootsStore) -> eyre::Result<CliOutput> {
        // A small local settings read/write and path check.
        Ok(CliOutput::facet(
            store.add(Path::new(&self.path))?,
            Some(std::time::Duration::from_secs(1)),
        ))
    }
}
