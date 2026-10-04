//! Logging flag aliases share the same filter and validation behavior.

use teamy_cancellation::CancellationToken;
use teamy_instagram::cli::Cli;
use teamy_instagram::logging_init;
use tracing_subscriber::EnvFilter;

fn parse_cli(arguments: &[&str]) -> Result<Cli, figue::DriverError> {
    figue::Driver::new(
        figue::builder::<Cli>()
            .expect("CLI schema should be valid")
            .cli(|cli| cli.args(arguments.iter().copied()).strict())
            .help(|help| help.program_name("teamy-instagram"))
            .build(),
    )
    .run()
    .into_result()
    .map(|output| output.value)
}

#[test]
fn canonical_and_alias_preserve_the_same_full_filter() {
    let filter = "warn,teamy_instagram=debug,hyper=error";
    let canonical = parse_cli(&["--log-filter", filter, "archive", "validate", "fixture.zip"])
        .expect("canonical filter should parse");
    let alias = parse_cli(&["--log-level", filter, "archive", "validate", "fixture.zip"])
        .expect("alias should accept full filter directives");

    assert_eq!(canonical.global_args, alias.global_args);
    assert_eq!(alias.global_args.log_filter.as_deref(), Some(filter));
    EnvFilter::builder()
        .parse(alias.global_args.log_filter.as_deref().expect("filter"))
        .expect("the alias should retain valid EnvFilter syntax");
}

#[test]
fn both_spellings_work_after_nested_subcommands() {
    for flag in ["--log-filter", "--log-level"] {
        let cli = parse_cli(&["archive", "validate", "fixture.zip", flag, "info"])
            .expect("global filter should parse after nested subcommands");
        assert_eq!(cli.global_args.log_filter.as_deref(), Some("info"));
    }
    let default =
        parse_cli(&["archive", "validate", "fixture.zip"]).expect("default arguments should parse");
    assert_eq!(default.global_args.log_filter, None);
}

#[test]
fn root_help_lists_alias_and_nested_help_accepts_both_spellings() {
    let error = parse_cli(&["--help"]).expect_err("root help should return a help outcome");
    let figue::DriverError::Help { text, suggestion } = error else {
        panic!("expected root help outcome, received {error}");
    };
    assert!(suggestion.is_none(), "explicit help should have no error");
    assert!(text.contains("--log-filter"), "{text}");
    assert!(text.contains("aliases: log-level"), "{text}");

    for flag in ["--log-filter", "--log-level"] {
        let error = parse_cli(&["archive", "validate", "fixture.zip", flag, "info", "--help"])
            .expect_err("nested help should return a help outcome");
        let figue::DriverError::Help { text, suggestion } = error else {
            panic!("expected nested help outcome, received {error}");
        };
        assert!(suggestion.is_none(), "explicit help should have no error");
        assert!(text.contains("teamy-instagram archive validate"), "{text}");
        assert!(
            text.contains("Strictly parse supported JSON sections"),
            "{text}"
        );
    }
}

#[test]
fn debug_conflicts_with_canonical_filter_and_alias_before_logging_starts() {
    for flag in ["--log-filter", "--log-level"] {
        for arguments in [
            vec![
                "--debug",
                flag,
                "info",
                "archive",
                "validate",
                "fixture.zip",
            ],
            vec![
                "archive",
                "validate",
                "fixture.zip",
                flag,
                "info",
                "--debug",
            ],
        ] {
            let cli = parse_cli(&arguments).expect("flags should parse into shared log state");
            let error = logging_init::init_logging(&cli.global_args, CancellationToken::new())
                .expect_err("debug and an explicit filter must conflict before initialization");
            assert_eq!(error.to_string(), "cannot specify log filter with --debug");
        }
    }
}
