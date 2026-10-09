# Command completion budgets

Each successful leaf returns an explicit `elapsed_duration_warn_threshold`. Dispatch forwards it, and the entrypoint measures from immediately before invocation through output. It warns only when elapsed duration is strictly greater than the returned threshold, including when rendering or writing fails. Failed invocation has no receipt. Budget metadata never enters serialized reports and validation failure is still emitted after its report.

- Roots add/list/remove: 1 second for local settings I/O and path inspection.
- Archive inventory/latest: 2 seconds plus 2 ms per configured maximum inventory entry, covering traversal and report rendering.
- Archive validate/activity: 3 seconds plus 100 ms per ceiling MiB of permitted recognized JSON and 1 ms per permitted ZIP directory entry. These use effective validation limits, including CLI overrides, independently of actual elapsed time.

Duration arithmetic saturates. These are completion diagnostics, not watchdogs. No command is exempt because all current archive commands enforce finite entry/byte limits. Existing operation-specific tracing remains separate.
