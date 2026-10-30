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
