# Teamy Instagram

A reusable Rust library and Facet/Figue CLI for managing local Instagram export archives, validating selected JSON sections, and listing reel activity. MPL-2.0.

```powershell
cargo run -- roots add <archive-directory>
cargo run -- roots list
cargo run -- roots remove <archive-directory>
cargo run -- archive inventory
cargo run -- archive latest --date-basis export-filename
cargo run -- archive latest --date-basis file-modified
cargo run -- archive validate <zip-path>
cargo run -- archive activity <zip-path>
cargo run -- archive activity <zip-path> --summary
cargo run -- archive activity <zip-path> --self-label "Synthetic self label"
cargo run -- --output-format json archive activity <zip-path>
```

The ZIP is opened read-only. Only recognized JSON entries are decompressed, one bounded entry at a time; nothing is extracted. Defaults are 32 MiB per entry, 512 MiB total recognized bytes, and a bounded entry count. Override byte limits with `--max-entry-bytes` and `--max-total-bytes`. Cancellation is checked between entries and during reads; a single bounded Facet parse runs synchronously.

## Roots and archive selection

`roots add` canonicalizes an existing directory and persists a deduplicated root list in application local appdata. `roots list` reads settings without changing them. `roots remove` accepts an existing directory or its exact stored canonical path if the directory is gone. Invalid existing settings are preserved and reported with fixed error codes. Use `roots --configuration-path <settings-path> add <archive-directory>` to override the settings file; `archive inventory` and `archive latest` accept the same `--configuration-path` flag.

Inventory recursively finds files named `instagram-*.zip` under configured roots without opening their contents. A service filename does not validate a ZIP. Traversal is bounded to 100,000 filesystem entries by default and a fixed directory-depth cap; symlinks are skipped and reported. Overlapping roots are deduplicated. `--shallow` limits traversal to each root's direct entries; `--max-entries` changes the entry cap. Unreadable entries and incomplete traversal remain explicit.

Latest selection requires `--date-basis export-filename` or `--date-basis file-modified`. The filename basis uses a valid calendar date from the service filename; modification time is a separate filesystem observation. Missing dates are counted and skipped without switching bases. Equal dates select the lexicographically smallest canonical path. The result reports its date basis and whether inventory was incomplete; no internal export timestamp is inferred.

Root, inventory, and latest outputs contain local paths for your use. Keep these outputs and the appdata settings file private. Paths and filesystem error details are excluded from logs and diagnostics.
## Reel activity

`archive activity` emits normalized reel identities, evidence occurrences, and separate coverage for six activities:

| Activity | Evidence source |
| --- | --- |
| Seen | Reel links in viewed-story records or the watched-video section under ads information, with these sources kept distinct. |
| Liked | Reel links in liked-post records. |
| Bookmarked | Reel links in saved-post records. |
| Sent | Reel links shared in messages sent by an explicitly mapped exporter label. |
| Received | Reel links shared by another sender when exporter identity is resolved. |
| Reacted | The mapped exporter's reaction on a message containing a reel reference. |

A heart on a message is message-reaction evidence, separate from liking its reel. Other participants' reactions do not count as the exporter's reactions. A message-share timestamp is stored in milliseconds. An exported reaction timestamp, when present, is retained separately with an unspecified unit rather than inferring one from its magnitude. An absent reaction timestamp remains unknown.

Supply `--self-label` only for an exact exported sender/actor label you have confirmed as yourself. Repeat it for explicitly confirmed aliases. Matching is case-sensitive. No name or username is automatically treated as the exporter. Without a usable mapping, shared-message and reaction evidence remains unresolved and the affected activity coverage is `Ambiguous`. Identical participant display labels cannot establish which person acted.

Coverage is `Present`, `Absent`, `Unsupported`, or `Ambiguous` per activity. Missing optional sections are valid and do not fail parsing; `Absent` describes available export evidence, not proof that an activity never occurred. Schema failures, unsupported reference forms, and unresolved identity stay visible. Viewed stories and watched-video records are partial observations: neither establishes duration, completion, or a complete reel watch history. A report never claims complete export coverage.

Reel identity is deduplicated by a validated shortcode and canonical reel URL. Individual evidence occurrences remain separate, including repeated URLs and repeated events. Repeated href/value representations are evidence occurrences; their count does not establish distinct user actions. Per-activity `distinct_reel_count` counts normalized reel identities, `source_event_count` groups evidence by its export record, message, or reaction indices, and `occurrence_count` retains every URL representation. Provenance uses reader entry indices, record/message/reaction indices, field locations, and source timestamps. Post links are not assumed to be reels, and opaque reel-share tokens are not fabricated into reel IDs. Activity expansion is capped at 100,000 evidence events per entry and 1,000,000 per report. Supported shortcodes are bounded to 128 ASCII bytes; rejected candidates remain visible in reference counts.

Reel URLs in an explicitly labeled `Caption` are retained under `incidental_references`; they do not establish that the exporter liked, saved, or viewed that referenced reel. These are excluded from activity identity and event counts. `reference_counts` covers all scanned URL occurrences, including incidental ones.

Detailed activity output contains reel IDs and canonical URLs from your export. Keep it local. Use `--summary` for aggregate counts and coverage without reel IDs, URLs, participant labels, message contents, or source paths. Logs never contain export contents or command arguments.

## What validation means

`archive validate` checks selected schemas without extracting activity. Supported schemas cover saved posts, liked posts, liked comments, viewed stories, watched videos under ads information, and message threads with nested reactions. The current generic record format uses recursive `label_values`. Older wrapper formats are not claimed as supported. All typed containers use `#[facet(deny_unknown_fields)]` in every build. An event-only preflight follows the Facet shapes to reject scalar coercions, required-field nulls, malformed document boundaries, and nesting beyond 128 schema frames. Empty observed `media`, label-value vector containers, and `magic_words` arrays do not establish their future element schemas.

The validation report identifies entries by ZIP-reader index and category, never by private path. It distinguishes parsed entries, schema/read/limit failures, unsupported JSON and other entries, media, and directories. ZIP central-directory metadata is indexed by the reader; payload limits do not bound that metadata or typed-model allocation overhead. The reader coalesces duplicate entry names, so coverage describes reader-exposed entries and does not verify the raw ZIP structure. Typed item counts are included for parsed entries. Missing optional sections are valid: export selections differ. Unsupported entries stay unsupported; successful selected-schema validation does not establish complete export coverage or authenticity. Unknown data is never silently reported as parsed.

Recognized-entry failures return a nonzero exit status **after** emitting either report. Unsupported sections and identity ambiguity alone do not fail validation. Invalid ZIP files and cancellation return nonzero status. Raw JSON parser diagnostics are replaced with safe error codes, so values, URLs, participant names, and entry paths are absent from validation reports and logs. Debug mode remains strict.

Output uses the template's common rendering layer: text in a terminal, JSON when redirected, or explicit `--output-format`. CSV may not represent nested reports; use JSON for automation. Structured logs are optional via `--log-file`; keep outputs local and do not commit exports or logs.

## Library and development

`teamy_instagram::archive::validate_archive` accepts a local path, explicit validation limits, and a cancellation token. `teamy_instagram::activity_report::extract_archive` additionally accepts an explicit `identity::ExporterIdentity` and returns activity evidence with coverage; `aggregate_summary()` removes reel details. Raw export types live under `models`; archive processing and activity extraction are separate from CLI dispatch, suitable for later composition as `teamy instagram`.

```powershell
./check-all.ps1
```

The inherited gate checks nightly formatting, Clippy with all targets/features, lint policy, build, and non-Tracy tests. Fixtures are synthetic; tests cover strict schemas, resource bounds, cancellation, CLI privacy, root persistence, explicit latest selection, normalized references, repeated evidence, activity classification, and unresolved identity. Dependencies use one pinned compatible Facet family; there are no machine-local crate patches.

Scaffolded from the current local [teamy-rust-cli](https://github.com/TeamDman/teamy-rust-cli) template. Its output, cancellation, tracing, Windows resources, fuzz tests, and quality gate were retained and adapted. Third-party project code was not copied.

## Next slices

Later metadata work can retain creator/title observations separately from stable reel identity. A future sync plan should be a reviewable dry run: stable creator/reel/ID folders and info JSON, followed only when requested by media and transcript work. This command downloads no media and produces no transcripts. No MFT/USN dependency or default perceptual deduplication policy is introduced.
