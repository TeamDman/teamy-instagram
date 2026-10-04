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
}

impl ArchiveArgs {
    /// # Errors
    /// Returns an error on archive failure or cancellation.
    pub fn invoke(
        self,
        cancellation: &CancellationToken,
    ) -> std::future::Ready<eyre::Result<CliOutput>> {
        std::future::ready(match self.command {
            ArchiveCommand::Validate(args) => args.invoke(cancellation),
        })
    }
}
