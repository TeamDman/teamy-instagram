use crate::catalog::RootsStore;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[expect(
    clippy::empty_structs_with_brackets,
    reason = "Facet and Figue require the empty argument object shape"
)]
pub struct RootsListArgs {}

impl RootsListArgs {
    /// # Errors
    /// Returns a fixed code for unreadable or invalid settings.
    pub fn invoke(self, store: &RootsStore) -> eyre::Result<CliOutput> {
        Ok(CliOutput::facet(store.list()?))
    }
}
