Each subcommand must have its own directory module.
Each subcommand implementation must live in a new `{}_{}_{}_cli.rs` file that `mod.rs` re-exports to ensure fuzzy finders can find the file easily.
Export privacy: only synthetic fixtures belong in this public repository. Never copy local exports, real identifiers, message contents, URLs, participant names, personal absolute paths, or raw parser errors into source, fixtures, documentation, or committed logs.
Use Facet, not Serde. Put domain/model types in separate files where practical. Validation must retain strict nested unknown-field checks in all build profiles and report unsupported sections explicitly. Do not infer message direction without explicit exporter identity mapping; message reactions are separate evidence from reel likes.
