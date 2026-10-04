pub mod archive;
pub mod cli;
pub mod command_timing;
pub mod logging_init;
pub mod models;
#[cfg(windows)]
mod windows_startup;

use crate::cli::Cli;
use crate::command_timing::CommandTimer;
use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use std::io::IsTerminal;
use std::io::{self};
use teamy_cancellation::CtrlCHandler;

/// Version string combining package version, git repository metadata, and build time.
fn version() -> String {
    let built_at = option_env!("BUILD_TIMESTAMP_UNIX")
        .and_then(|value| value.parse::<i64>().ok())
        .and_then(|timestamp| DateTime::<Utc>::from_timestamp(timestamp, 0))
        .map_or_else(
            || "unknown build time".to_string(),
            |timestamp| {
                timestamp
                    .with_timezone(&Local)
                    .format("%Y-%m-%d %H:%M:%S %Z")
                    .to_string()
            },
        );

    format!(
        "{} (repo {}, branch {}, rev {}, worktree {}, built {})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_REPOSITORY_URL"),
        env!("GIT_BRANCH"),
        env!("GIT_REVISION"),
        env!("GIT_WORKTREE_STATUS"),
        built_at,
    )
}

/// Entrypoint for the program.
///
/// # Errors
///
/// This function will return an error if `color_eyre` installation, CLI parsing, logging initialization, command execution, or command output rendering fails.
///
/// # Panics
///
/// Panics if the CLI schema is invalid (should never happen with correct code).
pub fn main() -> eyre::Result<()> {
    // Install color_eyre for better error reports
    color_eyre::install()?;
    let cancellation_token = CtrlCHandler::default().install()?;

    #[cfg(windows)]
    {
        // Enable ANSI support on Windows
        // This fails in a pipe scenario, so we ignore the error
        let _ = windows_startup::enable_ansi_support();

        // Warn if UTF-8 is not enabled on Windows
        #[cfg(windows)]
        windows_startup::warn_if_utf8_not_enabled();
    };

    // Parse command line arguments using figue
    // unwrap() is figue's intended CLI entry behavior:
    // it exits with proper codes for --help/--version/completions/parse-errors.
    let version = version();

    let cli: Cli = figue::Driver::new(
        figue::builder::<Cli>()
            .expect("schema should be valid")
            .cli(move |cli| cli.args_os(std::env::args_os().skip(1)).strict())
            .help(move |help| {
                help.version(version)
                    .include_implementation_source_file(true)
                    .include_implementation_github_url(
                        "TeamDman/teamy-instagram",
                        env!("GIT_REVISION"),
                    )
            })
            .build(),
    )
    .run()
    .unwrap();

    let _stop_after_duration_thread = cli
        .global_args
        .stop_after
        .start_stop_after_duration_thread(cancellation_token.clone())?;

    // Initialize logging
    logging_init::init_logging(&cli.global_args, cancellation_token.clone())?;

    // Measure command work after logging setup, including output and early returns.
    let command_name = cli.command.name();
    let _command_span = tracing::info_span!("command", command = command_name).entered();
    let mut command_timer = cli
        .command
        .expected_duration()
        .map(|expected| CommandTimer::new(command_name, expected));
    if let Some(timer) = &mut command_timer {
        timer.checkpoint("dispatch");
    }

    // Invoke whatever command was requested and render its output once at the top level
    let requested_output_format = cli.global_args.output_format;
    let output = cli.invoke(cancellation_token.clone())?;
    if let Some(timer) = &mut command_timer {
        timer.checkpoint("dispatch");
    }
    cancellation_token.bail_if_cancelled()?;
    let stdout = io::stdout();
    let output_is_terminal = stdout.is_terminal();
    if let Some(timer) = &mut command_timer {
        timer.checkpoint("output");
    }
    output.emit_to(
        &mut stdout.lock(),
        requested_output_format,
        output_is_terminal,
    )?;
    cancellation_token.bail_if_cancelled()?;
    Ok(())
}
