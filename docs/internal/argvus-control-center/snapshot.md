# Snapshot implementation

## Ownership and entry points

`argvus-control-center-snapshot` owns the typed integration state, Snapper backend
and TUI page. `argvus-control-center` registers it through the independent
`snapshot` feature, included in `full`. `stable` is unchanged. Home, global search
and `argvus control-center snapshot` open the same page and start its refresh.
The breadcrumb uses `HomeCardId::System`, matching the Home catalog.

The CLI dispatcher already forwards Control Center arguments to the private
executable, so it needs no additional route. Storage, Boot, Packages and Diagnostics
retain their existing responsibilities. The only affected Arch package is
`argvus`; `snapper` is an optional runtime dependency. `util-linux` and Polkit
are existing required dependencies.

## State and process boundary

A first background job detects the root mount through `/usr/bin/findmnt` JSON.
Non-Btrfs roots stop there. A subsequent job checks `/usr/bin/snapper --help`,
queries JSON configurations, verifies `SUBVOLUME=/` and `FSTYPE=btrfs`, and loads
the selected configuration. Multiple matching configurations require selection.
Snapper's JSON output is config-keyed for snapshots and has a `configs` array
for `list-configs`; `get-config` is a key/value object. Protection flags must be
explicit JSON booleans. Malformed or absent metadata fails closed.

`Integration` separates incompatible filesystem, missing executable, no root
configuration, configuration choice and a ready list. `ErrorKind` separates
filesystem detection, permission, authentication, timeout, compatibility,
validation and backend failures. Progress is held separately for detection,
loading and changes. Rendering consumes cached state and never probes the host.
Only one page job can run at a time. Refresh preserves selection by snapshot
number. A replaced configuration can be rediscovered on the next refresh.

All subprocesses use `ProcessRunner::run`, structured arguments, closed stdin,
`LC_ALL=C.UTF-8`, a 1 MiB limit per stream and deadlines (15 seconds for queries,
120 seconds for changes). The outer helper request allows 90 seconds for an
inspection or 180 seconds for a change including authentication. Stderr shown in
the UI is terminal-sanitized and limited to 512 characters. The shared job runner
handles cooperative process cancellation; an interrupted mutation is never
retried automatically, since it may have completed. A read-back refresh follows
both successful and failed mutation results.

## Restricted privilege interface

Comparison adds restricted `status <config> <origin> <destination>` and
`diff <config> <origin> <destination> <path>` operations. The privileged backend
rechecks the root configuration and snapshot IDs, and the diff path must appear
in a freshly parsed status result. Status is limited to 16 MiB; diff is limited
to 256 KiB. The fake ProcessRunner tests exercise argument boundaries.

The existing `SystemSettingsOperation` builds the helper chain. Ordinary queries
run without elevation. On denied access, `a — Authorize` explicitly enables
queries through Polkit; opening the page never triggers authentication by itself.
No credentials are cached by the page, and later requests may require another
Polkit prompt.

The Settings crate enables its optional `snapshot` dependency when the Control
Center feature is selected. Its existing argument validator accepts only:

- `system-settings snapshot inspect [config]`
- `system-settings snapshot create <config> <description>`
- `system-settings snapshot delete <config> <positive-number>`

The existing `ensure_root` mechanism elevates this command, which returns before
starting the TUI. The snapshot handler independently requires effective UID 0;
it never trusts the inherited elevation marker alone. Commands use fixed
`/usr/bin` paths. Successful helper execution emits one serialized Rust `Result`
containing typed integration state, a created number or a structured backend
error. Authentication errors before entering the handler retain process status.

Both UI and helper validate the request. Before mutation, the helper repeats root,
configuration and snapshot queries. Deletion requires membership in the selected
configuration, a positive integer and neither active nor default. Snapper remains
the final authority if another process changes state between check and operation.
Creation passes one printable description of at most 256 characters and invokes
`create --type single --description <value> --print-number`. Deletion passes one
canonical integer to `delete`; ranges and lists are rejected.

No Polkit rules, setuid binaries, direct writes to snapshot directories or config
files, installation, configuration creation, cleanup, restore or rollback are added.

## Validation and reference

Tests inject a recording fake `ProcessRunner`; the existing `testing` feature
also prevents application tests from starting real tools. Coverage includes root
parsing, errors, configuration matching, JSON, argument boundaries, protected
snapshots, confirmation, translations, rendering, selection and all entry points.
Builds cover default, no features, snapshot alone, stable and full without snapshot.

The host used for implementation has an ext4 root and no Snapper executable.
Interactive PTY checks verified the Portuguese and English incompatible-filesystem
screens, disabled create/delete keys, contextual help, refresh and clean exit.
Actual Btrfs mutations and Polkit interaction require a disposable Btrfs system;
manual steps are in the bilingual Control Center user pages.

Syntax and JSON shape were checked against the [Arch manual](https://man.archlinux.org/man/snapper.8.en)
and the official Snapper client sources (`cmd-list.cc`, `cmd-list-configs.cc`,
`cmd-get-config.cc`) in the [upstream repository](https://github.com/openSUSE/snapper/tree/master/client/snapper).

## Selective restore verification (S1)

After undochange, verification reads only the selected regular file in 64 KiB
chunks and compares its contents, mode, UID and GID with the destination snapshot.
Missing files on both sides compare equal. Unsafe paths or files changed during
verification fail closed, retaining the safety snapshot. No root-wide status is
requested after restore. The preview still invokes a bounded Snapper diff against
the live system; Snapper may perform filesystem work before producing that output.
Real Btrfs restore and Polkit behavior still require a disposable system.

## Bounded comparison responses (S2)

Comparisons retain at most 10,000 paths and an estimated 8 MiB JSON budget.
The response includes `truncated` and `total_seen`; the page shows the retained
count and only offers actions on retained paths. The count describes all parsed
paths, not just the retained prefix. Each wire entry contains only `path` and
`indicators`; deserialization validates indicators and derives all flags.
The helper transport permits 16 MiB. A Snapper output above its hard 16 MiB
limit returns the translated OutputLimit error, because the process runner does
not retain partial output on overflow. The UI and helper must be upgraded together.

## Btrfs identity for selective restore (S3)

Restore preview verifies that root and source are Btrfs with fstatfs, then compares
filesystem UUIDs using BTRFS_IOC_FS_INFO on open descriptors. It does not compare
st_dev or statfs f_fsid: both may distinguish subvolumes. The UUID probe is
injectable through RestorePaths for tests. Non-Btrfs paths, failed probes and
mismatched UUIDs fail closed. Source path ownership and symlink checks remain.

## Privileged helper operands (H1)

Settings helper operands are validated again at execution. Timezone, keyboard
and DNS connection operands cannot begin with a dash. Service operations accept
unit names from a restricted character set, reject paths and leading dots, and
place a systemctl option terminator before the unit. timedatectl and localectl
also receive an option terminator before the validated operand.

## Follow-up validation and process lifecycle

The real_btrfs_subvolume_identity ignored integration test passed in a disposable
QEMU/KVM guest running the local 6.18.57-2-lts kernel. Different subvolume st_dev
values shared the expected filesystem UUID. A distinct Btrfs filesystem and procfs
were rejected; content verification was also exercised. Full Snapper/Polkit
interaction remains unverified.

The core process runner terminates remaining members of a managed command's
process group after its parent exits, including commands without timeouts, so
inherited output pipes cannot keep the UI job waiting indefinitely. Persistent
services must be started by a service manager.

Live process output is drained as bytes: invalid UTF-8 cannot stop a reader and
block the command. Raw captured output preserves the original bytes; live rows
use lossy UTF-8 conversion and split on both LF and CR for progress updates.
Interrupted reads are retried. Output retention limits are a separate concern
tracked by audit item C8.

Restore review uses a read-only viewport independent of the comparison cursor.
Diff and review scroll offsets are bounded by the available text; moving beyond
the last line or character cannot keep scrolling through empty content.

## Automatic snapshot retention (S5)

Pre-upgrade and selective-restore safety snapshots are created with
`--cleanup-algorithm number`. Snapper number retention settings and its cleanup
execution determine when they become eligible for deletion. Manual creation
does not force an algorithm. Existing snapshots without a cleanup algorithm
are not relabeled by this change.

## Unsupported comparison paths (S6)

Comparison responses include an `unsupported` record count. Invalid UTF-8,
control characters and unsupported path components are excluded from actionable
changes while valid records remain visible. The UI displays a translated count.
Unframed continuation lines discard the preceding apparent path conservatively.
The line-delimited Snapper status format cannot fully disambiguate filenames
containing newlines that imitate complete status records.

## Shared comparison execution (S7)

Status command construction and parsing are shared by comparison and path
validation; diff decoding is shared by public diff and restore preview. Public
diff validates fresh snapshot IDs and status membership without building the UI
comparison prefix. Restore preview issues one inspection, one status and one
path-filtered diff. Status remains a filesystem comparison because Snapper does
not expose a per-file status operand; no cross-request validation cache is used.

## Process cancellation result (C5)

Core ProcessOutput distinguishes cooperative `cancelled` from deadline
`timed_out`; a cancelled process has no exit status. Network editor jobs keep
their handle until completion, and closing the editor while work is pending
does not discard its result. Network refresh cancellation targets only read jobs.

## Terminal text normalization (C9)

The shared core sanitizer uses explicit states for CSI, OSC, DCS, APC, PM and
SOS, including C1 forms, and strips bidi/invisible formatting controls from
external text. CSI accepts the complete ASCII final-byte range. Unterminated
OSC strings are discarded through the next newline or EOF; parsing then resumes
at the next line. Other control strings suppress multiline payloads until ST/EOF. Plain Unicode text, tabs and newlines are preserved.
Each call processes a complete string independently.

## Mount boundary path encoding (S8)

Mountinfo fields are read as Unix bytes. Octal escapes decode to bytes and
OsString/PathBuf preserve UTF-8 and non-UTF-8 mountpoint names. Restore boundary
checks compare those paths without converting byte escapes to Unicode codepoints.
Malformed/overflowing escapes and NUL paths are rejected by the decoder.

## Lock contention messages (S9)

Package transaction and selective restore locks have distinct typed contention
errors and translated messages. Acquisition order remains package then restore.
A refusal of the second lock releases the first guard; no Snapper mutation is
started. Non-contention flock failures retain their actual OS error.

## Retention configdata and fallback provenance (S10)

UI/helper configdata stays a validated space-delimited string, parsed by one
shared function. The Snapper executor passes each KEY=VALUE pair as a separate
argument and verifies read-back. Missing settings are labeled as ARGVUS fallbacks,
not values read from the installed creation template. The legacy wire origin
SnapperDefault identifies that fallback; explicit get-config values take priority.

## Snapshot UI state (S11)

Configuration selection starts at the first entry and clears the previous
configuration context. Failed retention writes invalidate displayed values and
trigger a fresh read for the selected configuration while preserving the error.
Text and styled rows are cached between state changes, keyed by language; idle
polls reuse them. Input, completed replies, reloads and theme changes invalidate
the cache. Scroll offsets remain bounded by the rendered content.

## Package independence

Package operations do not inspect the filesystem or Snapper configuration and
never create pre-update snapshots. Full upgrades execute `pacman -Syyu` and
individual upgrades execute `pacman -S <package>` through the privileged helper.
Snapshot inspection and configuration selection belong only to the Snapshots page.

## Package mutation exclusion (H6)

The settings package dispatcher holds the shared package-update snapshot lock
for every package operation, including install/remove/reinstall/downgrade,
metadata refresh, cache cleanup, mirror and repository edits. Selective restore
uses the same file and nonblocking exclusive flock. The guard spans the package command and is released on errors. This lock only
prevents concurrent ARGVUS mutations; it imposes no filesystem or snapshot
requirements. External pacman/AUR processes do not participate in this ARGVUS lock.

The package guard explicitly issues `LOCK_UN` before closing its descriptor.
Closing alone can retain the lock through a duplicate inherited by a concurrent
fork, because `O_CLOEXEC` takes effect only at exec. A regression test retains a
duplicated descriptor while dropping the guard and reacquiring the lock, then
checks that closing the old duplicate cannot release the next transaction's lock.

## Application lifetime during mutations (C10)

The parent application consults has_pending_action across every compiled page
before accepting q/Ctrl+C. Snapshot Changing/Restoring jobs block exit until
their result is polled; ordinary snapshot inspection/comparison jobs do not.
Other pages expose action state, including package transactions and network
editor jobs. A refused exit preserves the job and shows a translated footer;
completion does not automatically close the application. Mixed-purpose slots
are conservatively retained. This is a keyboard-exit guard, not a guarantee
against external signals or loss of the terminal session.
