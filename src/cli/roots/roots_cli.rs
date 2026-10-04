use crate::catalog::RootsStore;
use crate::cli::output::CliOutput;
use crate::cli::roots::add::RootsAddArgs;
use crate::cli::roots::list::RootsListArgs;
use crate::cli::roots::remove::RootsRemoveArgs;
use arbitrary::Arbitrary;
use facet::Facet;
use figue::{self as args};
use std::path::PathBuf;
use teamy_cancellation::CancellationToken;

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[facet(rename_all = "kebab-case")]
pub struct RootsArgs {
    /// Optional settings file; otherwise use the application local appdata.
    #[facet(args::named)]
    pub configuration_path: Option<String>,
    #[facet(args::subcommand)]
    pub command: RootsCommand,
}

#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum RootsCommand {
    /// Register an existing directory by its canonical path.
    Add(RootsAddArgs),
    /// List configured roots without changing settings.
    List(RootsListArgs),
    /// Remove a root, including an exact stored path that no longer exists.
    Remove(RootsRemoveArgs),
}

impl RootsArgs {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self.command {
            RootsCommand::Add(_) => "roots add",
            RootsCommand::List(_) => "roots list",
            RootsCommand::Remove(_) => "roots remove",
        }
    }

    /// # Errors
    /// Returns fixed settings errors or cancellation.
    pub fn invoke(
        self,
        cancellation: &CancellationToken,
    ) -> std::future::Ready<eyre::Result<CliOutput>> {
        std::future::ready(self.invoke_sync(cancellation))
    }

    fn invoke_sync(self, cancellation: &CancellationToken) -> eyre::Result<CliOutput> {
        cancellation.bail_if_cancelled()?;
        let store = match self.configuration_path {
            Some(path) => RootsStore::new(PathBuf::from(path)),
            None => RootsStore::in_appdata()?,
        };
        match self.command {
            RootsCommand::Add(args) => args.invoke(&store),
            RootsCommand::List(args) => args.invoke(&store),
            RootsCommand::Remove(args) => args.invoke(&store),
        }
    }
}
