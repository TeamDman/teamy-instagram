use crate::cli::archive::activity::ActivityArgs;
use crate::cli::archive::inventory::InventoryArgs;
use crate::cli::archive::latest::LatestArgs;
use crate::cli::archive::validate::ValidateArgs;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use teamy_cancellation::CancellationToken;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
pub struct ArchiveArgs {
    #[facet(args::subcommand)]
    pub command: ArchiveCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum ArchiveCommand {
    /// Strictly parse supported JSON sections and report partial coverage.
    Validate(ValidateArgs),
    /// List normalized reel evidence and report per-activity coverage.
    Activity(ActivityArgs),
    /// Inventory service ZIP files under configured roots.
    Inventory(InventoryArgs),
    /// Select the latest inventoried archive using an explicit date basis.
    Latest(LatestArgs),
}

impl ArchiveArgs {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self.command {
            ArchiveCommand::Validate(_) => "archive validate",
            ArchiveCommand::Activity(_) => "archive activity",
            ArchiveCommand::Inventory(_) => "archive inventory",
            ArchiveCommand::Latest(_) => "archive latest",
        }
    }

    /// # Errors
    /// Returns an error on archive failure or cancellation.
    pub fn invoke(
        self,
        cancellation: &CancellationToken,
    ) -> std::future::Ready<eyre::Result<CliOutput>> {
        std::future::ready(match self.command {
            ArchiveCommand::Validate(args) => args.invoke(cancellation),
            ArchiveCommand::Activity(args) => args.invoke(cancellation),
            ArchiveCommand::Inventory(args) => args.invoke(cancellation),
            ArchiveCommand::Latest(args) => args.invoke(cancellation),
        })
    }
}
