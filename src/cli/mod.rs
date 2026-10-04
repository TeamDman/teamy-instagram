pub mod archive;
pub mod facet_shape;
pub mod global_args;
pub mod output;

use crate::cli::archive::ArchiveArgs;
use crate::cli::global_args::GlobalArgs;
use crate::cli::output::CliOutput;
use arbitrary::Arbitrary;
use eyre::Context;
use facet::Facet;
use figue::FigueBuiltins;
use figue::{self as args};
use std::time::Duration;
use teamy_cancellation::CancellationToken;

/// Validate selected typed JSON sections in a local Instagram export ZIP.
/// Unknown sections remain explicitly unsupported. Export data is never logged.
#[derive(Facet, Arbitrary, Debug)]
pub struct Cli {
    #[facet(flatten)]
    pub global_args: GlobalArgs,
    #[facet(flatten)]
    #[arbitrary(default)]
    pub builtins: FigueBuiltins,
    #[facet(args::subcommand)]
    pub command: Command,
}

impl PartialEq for Cli {
    fn eq(&self, other: &Self) -> bool {
        self.global_args == other.global_args && self.command == other.command
    }
}

impl Cli {
    /// # Errors
    /// Returns an error if the runtime or command fails.
    pub fn invoke(self, cancellation_token: CancellationToken) -> eyre::Result<CliOutput> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .wrap_err("failed to build runtime")?;
        runtime.block_on(self.command.invoke(cancellation_token))
    }
}

/// Commands for local Instagram exports.
#[derive(Facet, Arbitrary, Debug, PartialEq)]
#[repr(u8)]
pub enum Command {
    /// Inspect a compressed archive without extracting files.
    Archive(ArchiveArgs),
}

impl Command {
    #[must_use]
    pub const fn name(&self) -> &'static str {
        "archive validate"
    }

    /// Archive runtime depends on export size; entry reads have byte bounds.
    #[must_use]
    pub const fn expected_duration(&self) -> Option<Duration> {
        None
    }

    /// # Errors
    /// Returns an error on cancellation or command failure.
    pub async fn invoke(self, cancellation_token: CancellationToken) -> eyre::Result<CliOutput> {
        cancellation_token.bail_if_cancelled()?;
        match self {
            Self::Archive(args) => args.invoke(&cancellation_token).await,
        }
    }
}
