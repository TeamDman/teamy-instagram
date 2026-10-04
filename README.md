# Teamy Instagram

A reusable Rust library and Facet/Figue CLI for validating selected JSON sections in a **local** Instagram export ZIP. MPL-2.0.

```powershell
cargo run -- archive validate <zip-path>
cargo run -- --output-format json archive validate <zip-path>
cargo run -- --debug archive validate <zip-path>
```

The ZIP is opened read-only. Only recognized JSON entries are decompressed, one bounded entry at a time; nothing is extracted. Defaults are 32 MiB per entry, 512 MiB total recognized bytes, and a bounded entry count. Override byte limits with `--max-entry-bytes` and `--max-total-bytes`. Cancellation is checked between entries and during reads; a single bounded Facet parse runs synchronously.

## What validation means

Supported schemas cover saved posts, liked posts, liked comments, viewed stories, and message threads with nested reactions. The current generic record format uses recursive `label_values`. Older wrapper formats are not claimed as supported. All typed containers use `#[facet(deny_unknown_fields)]` in every build. An event-only preflight follows the Facet shapes to reject scalar coercions, required-field nulls, malformed document boundaries, and nesting beyond 128 schema frames. Empty observed `media` and `magic_words` arrays do not establish their future element schemas.

The coverage report identifies entries by ZIP-reader index and category, never by private path. It distinguishes parsed entries, schema/read/limit failures, unsupported JSON and other entries, media, and directories. ZIP central-directory metadata is indexed by the reader; payload limits do not bound that metadata or typed-model allocation overhead. The reader coalesces duplicate entry names, so coverage describes reader-exposed entries and does not verify the raw ZIP structure. Typed item counts are included for parsed entries. Missing optional sections are valid: export selections differ. Unsupported entries stay unsupported; successful selected-schema validation does not establish complete export coverage or authenticity. Unknown data is never silently reported as parsed.

Recognized-entry failures return a nonzero exit status **after** emitting the report. Unsupported sections alone do not fail validation. Invalid ZIP files and cancellation return nonzero status. Raw JSON parser diagnostics are replaced with safe error codes, so values, URLs, participant names, and entry paths are absent from reports and logs. Debug mode remains strict.

Output uses the template's common rendering layer: text in a terminal, JSON when redirected, or explicit `--output-format`. CSV may not represent nested reports; use JSON for automation. Structured logs are optional via `--log-file`; keep outputs local and do not commit exports or logs.

## Library and development

`teamy_instagram::archive::validate_archive` accepts a local path, explicit validation limits, and a cancellation token. Raw export types live under `models`; archive processing is separate from CLI dispatch, suitable for later composition as `teamy instagram`.

```powershell
./check-all.ps1
```

The inherited gate checks nightly formatting, Clippy with all targets/features, lint policy, build, and non-Tracy tests. Fixtures are synthetic; tests cover nested unknown fields, malformed and unsupported inputs, resource bounds, cancellation, and report privacy. Dependencies use one pinned compatible Facet family; there are no machine-local crate patches.

Scaffolded from the current local [teamy-rust-cli](https://github.com/TeamDman/teamy-rust-cli) template. Its output, cancellation, tracing, Windows resources, fuzz tests, and quality gate were retained and adapted. Third-party project code was not copied.

## Next slices

Archive-root configuration and inventory/latest selection are deferred. Inventory must expose its date basis explicitly (export filename date, file modification time, or internal timestamp); these are different observations.

Later normalization should preserve separate saved, liked, viewed, message-share, and message-reaction evidence with entry and record provenance. A message heart is a reaction to a message, not proof of liking its reel. `stories_viewed` has no duration/completion field and is not a complete watch history. Incoming/outgoing classification requires an explicit exporter identity mapping; names alone are insufficient. Stable reel IDs should use validated newtypes and retain metadata observations separately from mutable creator/title fields.

A future sync plan should be a reviewable dry run: stable creator/reel/ID folders and info JSON, followed only when requested by media and transcript work. This command downloads no media and produces no transcripts. No MFT/USN dependency or default perceptual deduplication policy is introduced.
