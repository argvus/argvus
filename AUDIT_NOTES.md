# Control Center audit progress

## S1 — implementation and regression coverage

Confirmed: the old backend accepted mismatched file contents when the fake status
was empty. `restore_rejects_content_mismatch_despite_empty_status` failed before
the fix (unexpected successful RestoreResult).

Verification now compares only the selected file, contents and mode/UID/GID,
using bounded memory and checking the opened file identity before and after reads.
Tests cover mismatch preserving safety snapshot #9, an unused >1 MiB global
status response, missing files and permission differences. The fake runner does
not perform undochange: success fixtures contain the expected restored bytes.

Preview diff remains bounded but its internal Snapper traversal cost is unknown.
No claim of real Btrfs validation is made. On a disposable Btrfs system:

```sh
stat -c '%d %n' / /.snapshots/1/snapshot
stat -f -c '%t %i %n' / /.snapshots/1/snapshot
# After restoring a disposable root-owned test file from snapshot 1:
cmp -- /.snapshots/1/snapshot/etc/argvus-audit-test /etc/argvus-audit-test
stat -c '%a %u %g %n' /.snapshots/1/snapshot/etc/argvus-audit-test /etc/argvus-audit-test
```

All original audit IDs have been processed below. P10 code validation passed;
real multi-GPU handoff and other explicitly recorded hardware checks remain pending.

S1 validation: 52 snapshot tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. Real Btrfs validation remains pending.

## S2 — bounded comparison and transport

Confirmed with 50,000 status lines: the initial regression failed because every
entry was retained. Comparison now caps retained paths at 10,000 and budgets
8 MiB for worst-case escaped JSON. Wire entries carry only path and indicators.
UI and helper share the new model; the retained count is explicitly shown when
truncated, and only retained entries are selectable. Status and helper limits
are 16 MiB; hard process overflow is a typed translated error (partial output is
unavailable from ProcessRunner). Round-trip and overflow regression tests added.

S2 final validation: cargo fmt --all -- --check, cargo clippy --workspace
--all-targets -- -D warnings, cargo test --workspace, argvus i18n validate and
git diff --check passed. Snapshot suite now has 55 passing tests. No commit
was created, per the updated instruction. Installed catalog command passed;
bilingual source rendering is also covered by the new UI test.

## S3 — Btrfs filesystem identity

Confirmed the validation defect with an injected UUID probe: different UUIDs on
the same tempdir device were accepted before the fix. Root and source now require
Btrfs f_type and matching filesystem UUIDs from BTRFS_IOC_FS_INFO. st_dev is no
longer compared. Probe failure rejects restoration. Tests inject the UUID probe;
tempdir tests do not claim to reproduce kernel subvolume behavior.

The proposed statfs f_fsid comparison was intentionally avoided: kernel versions
also encode the subvolume root ID there. Reference:
https://kernel.googlesource.com/pub/scm/linux/kernel/git/next/linux-next/+/refs/heads/akpm-base/fs/btrfs/super.c
The ioctl layout was checked against /usr/include/linux/btrfs.h.

Real-system validation remains required. In a disposable Btrfs VM, compare:

```sh
stat -c '%d %n' / /.snapshots/1/snapshot
sudo btrfs filesystem show --raw /
sudo btrfs filesystem show --raw /.snapshots/1/snapshot
```

Then use Control Center to preview and restore a disposable root-owned file from
that snapshot; confirm cmp and mode/UID/GID as described under S1. Also check that
a source on a distinct Btrfs filesystem or tmpfs is rejected. No mounts or
privileged mutations were performed during this task.

S3 validation: 58 snapshot tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. No commit created. No new catalog keys, runtime
configuration or packaging changes were required.

## Real-system validation availability

The current host reports ext4 for `/` (`findmnt --json --target / --output
FSTYPE,SOURCE,TARGET,OPTIONS`), no Btrfs mounts (`findmnt -t btrfs`), and no
`/.snapshots`. btrfs and snapper are installed. `sudo -n true` fails with
"a password is required". User namespaces are available but do not grant
initial-namespace privileges needed for loop devices and Btrfs mounting.
S1/S3 real filesystem and Polkit checks remain blocked by the missing disposable
Btrfs environment; unit tests are not a substitute for that validation.

## H1 — privileged option operands

Confirmed: the new regression failed on -Hfoo.service. Shared plain-argument
validation rejects empty, option-like and control-containing operands. Service
units use a restricted character set and reject paths and leading dots. systemctl,
localectl and timedatectl receive -- before untrusted operands. DNS execution
functions validate the connection again before filesystem or nmcli effects.
Regression tests cover direct executors and helper validation. No new visible
UI text, catalog keys or package changes.

H1 formatting and workspace clippy passed; settings suite passed (268 tests
before the second regression was added; both H1 regressions passed afterward).

H1 final validation: cargo test --workspace passed, as did cargo fmt --all
-- --check, cargo clippy --workspace --all-targets -- -D warnings and
git diff --check. No commits created.

## Follow-up: real Btrfs validation completed for S1/S3 primitives

The earlier environment blocker was worked around with an unprivileged QEMU/KVM
VM, the local kernel /boot/vmlinuz-linux-lts (6.18.57-2-lts) and a temporary initrd.
Only regular disk images under /tmp/argvus were used; no host disks were attached.
Two independent Btrfs filesystems and a read-only snapshot were created in guest.
The exact Rust test real_btrfs_subvolume_identity passed in the guest:
root st_dev=24, snapshot file st_dev=28; matching UUID accepted, distinct Btrfs
UUID and procfs rejected. Content comparison accepted matching bytes, rejected
changed bytes and accepted the restored bytes. Guest exit marker ARGVUS_VM_STATUS=0.
This supersedes the missing-environment blocker for these primitives only.
Full snapper undochange/Polkit UI interaction has NOT been verified.

The retained ignored test is reproducible in a disposable root guest by preparing:

```sh
mkdir -p /audit-root /audit-other
# Mount two distinct disposable Btrfs filesystems at those paths first.
btrfs subvolume create /audit-root/origin
printf 'version\n' > /audit-root/origin/file
btrfs subvolume snapshot -r /audit-root/origin /audit-root/snapshot
printf 'other\n' > /audit-other/file
cargo test -p argvus-control-center-snapshot real_btrfs_subvolume_identity -- --ignored --nocapture
```

## C1 — inherited pipes without timeout

Confirmed: completed_parent_without_timeout_closes_inherited_pipes failed after
2.00 s before the fix. Both run and run_live now terminate remaining members of
the managed process group after the parent exits, even without a timeout.
Managed commands must not leave workers running; long-lived services should be
started through their service manager. Regression tests cover both execution modes.

C1 final validation: 49 core unit tests and 2 process isolation tests passed;
cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings,
cargo test --workspace and git diff --check passed. The new Btrfs integration
test is ignored in ordinary runs and was explicitly executed successfully in
the disposable VM. Temporary VM artifacts were removed. No commits created.

## C2 — byte-oriented live pipe draining

Confirmed: live_reader_drains_non_utf8_and_progress_bytes failed before the fix:
the captured output was empty after 0xFF followed by a newline and 1 MiB of data.
The reader now uses byte buffers, keeps exact raw output, renders invalid UTF-8
lossily and emits progress rows on either CR or LF. Interrupted and WouldBlock
reads are retried; EOF and unrecoverable read errors finish draining.
Regression coverage includes a real child with invalid bytes followed by 1 MiB,
exact captured bytes, CR-delimited progress and an injected Interrupted error.
No new user-facing strings or packaging changes.

C2 final validation: 52 core unit tests and 2 process-isolation tests passed.
cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings,
cargo test --workspace and git diff --check passed. No commits created.

## C3 — Wi-Fi secrets through stdin

Confirmed: wifi_secret_is_sent_only_through_stdin failed before the fix because
the password was present in request.args. connect_wifi now uses nmcli --ask and
a newline-terminated ProcessRequest stdin payload. Empty, CR/LF and NUL-containing
secrets are rejected before spawning. Request Debug already hides stdin bytes.
Command and process errors redact the supplied secret before and after terminal
sanitization. Open networks keep closed stdin and do not enable --ask.
Regressions cover recorded argv/stdin/Debug, secret-bearing stderr, invalid secret
framing and open networks. No real Wi-Fi connection was changed during validation.
NetworkManager documents the --ask password prompt at:
https://networkmanager.dev/docs/api/latest/nmcli

C3 final validation: 89 network tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. English and Portuguese network documentation
updated; no new catalog keys or packaging changes. No commit created.

## C4 — stable Wi-Fi password target

Confirmed: missing_wifi_target_does_not_submit_secret_to_replacement_network
failed before the fix: confirming after removing cafe spawned an action for the
replacement row. The dialog now owns a cloned WifiNetwork, preserving SSID and
security through snapshot replacement. Confirmation checks that this identity
still exists and passes the captured network directly to the action job. Signal
and ordering changes do not change the target. Disappearance or security change
returns a translated error without starting the action. The current scan model
has no BSSID; identity is therefore SSID plus security, not a pinned access point.
Regressions cover disappearance, reordered scans, signal changes and security
changes, with the new error rendered in both supported locales.

C4 final validation: 92 network tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace,
argvus i18n validate and git diff --check passed. English and Portuguese
network documentation updated. No real Wi-Fi mutation, packaging change or
commit was performed.

## P1 — autostart IDs and Desktop Entry group

Confirmed by failing regressions: file_stem returned foo.desktop instead of foo,
and an action group changed the main entry name to Extra. IDs now strip one
.desktop suffix, while toggle accepts bare IDs and existing full filenames.
Parsing and flag rewrites are scoped to Desktop Entry; a missing Hidden key is
inserted before the next group. Action contents remain unchanged. Tests exercise
user entries and system copies with real filenames, enable/disable round trips,
unchanged system source, and action groups. Directory injection avoids changing
process-wide XDG variables or real user files. Invalid path IDs are rejected.

P1 final validation: 24 session tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. Control Center documentation updated in English
and Portuguese. No new UI strings, catalog keys, packaging changes or commits.

## P2 — service confirmation owns its scope

Confirmed: confirmation_captures_scope_for_identical_unit_names failed because
Pending contained only action and unit name. Pending now captures the user/system
scope at request time and confirmation passes it directly to run_action, without
searching the current inventory. Details preserve the selected scope as well as
the name; filtered list pages exclude the opposite scope. A missing detail target
cannot fall back to an unrelated selected row. Regression coverage includes the
same dbus.service in both scopes and reordered inventory from the Failed page.
No real service was started/stopped and no new UI strings were added.

P2 final validation: 19 services tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. English and Portuguese Control Center documentation
updated. No catalog, packaging or real service changes. No commits created.

## P3 — hierarchical findmnt inventory

Confirmed: nested_findmnt_includes_home_boot_and_efi failed with only / in the
result. parse_findmnt now traverses every children array in depth-first order
using an explicit stack. The existing mount conversion and pseudo-filesystem
filter are preserved. Tests cover /home, /boot, /boot/efi, read-only flags and
filesystem derivation. Diagnostics already consumes this shared filesystem
inventory and explicitly checks / and /home; no independent parser was added.

Real host verification passed: real_findmnt_tree_matches_flat_mount_inventory
compared tree output against --list output using the actual findmnt binary,
including target/source/fstype/options/readonly. The read-only integration test is
ignored in ordinary runs and can be repeated with:

```sh
cargo test -p argvus-control-center-storage real_findmnt_tree_matches_flat_mount_inventory -- --ignored
```

P3 final validation: 12 storage tests passed plus the explicitly executed real
findmnt integration test. cargo fmt --all -- --check, cargo clippy --workspace
--all-targets -- -D warnings, cargo test --workspace and git diff --check passed.
Control Center documentation updated in English and Portuguese. No new UI
strings, catalog keys, packaging changes or commits.

## S4 — read-only restore review and bounded scrolling

Confirmed: restore_review_render_does_not_follow_comparison_selection failed
before the fix; selecting row 9000 changed the review viewport. Reviews now use
the read-only renderer, independent of comparison selection. Both review and diff
scroll offsets are clamped to the actual line count and maximum character width
on input and before drawing. Tests cover rendering independence and upper bounds.
No new visible text or packaging changes.

S4 final validation: 60 snapshot tests passed, with 1 Btrfs integration test
ignored in the ordinary suite. cargo fmt --all -- --check, cargo clippy
--workspace --all-targets -- -D warnings, cargo test --workspace and
git diff --check passed. Internal snapshot documentation updated. No new
catalog keys, packaging changes or commits.

## S5 — automatic snapshot retention

Confirmed: both automatic_pre_upgrade_snapshot_uses_number_cleanup and
restore_safety_snapshot_uses_number_cleanup failed before the fix. Automatic
pre-upgrade and selective-restore safety snapshots now pass --cleanup-algorithm
number. The existing manual create interface stays unchanged and does not force
an algorithm; its exact argument regression remains in place. Both paths still
revalidate root/configuration/snapshots before mutation. No wire change, visible
string or packaging change is needed. Actual deletion depends on Snapper number
retention settings and running cleanup; historical snapshots without a cleanup
algorithm are not relabeled or deleted automatically by this fix.

S5 final validation: 62 snapshot tests passed, 1 Btrfs integration test ignored
in the ordinary suite. cargo fmt --all -- --check, cargo clippy --workspace
--all-targets -- -D warnings, cargo test --workspace and git diff --check passed.
Internal and bilingual user documentation updated. No real snapshots deleted
and no commits created.

## S6 — unsupported status records

Confirmed: comparison_keeps_valid_paths_around_unsupported_names failed with
status is not UTF-8. Comparison and helper path membership now parse records
individually, keeping valid names and counting unsupported records. The strict
parse_status validator remains for wire-entry decoding and parser contract tests.
Comparison carries unsupported alongside total_seen; both helper and UI use the
same updated type. The UI reports the count without offering unsupported action
rows. Byte control characters, invalid UTF-8 and invalid path components are
excluded. Duplicate valid paths still fail closed as ambiguous.

Unframed continuation records conservatively discard the preceding apparent path.
Snapper's line-delimited format cannot distinguish a filename containing a newline
followed by a perfectly valid-looking status record from separate filenames;
unsupported is therefore a count of rejected records, not a guaranteed count of
physical filenames. No claim of complete newline framing disambiguation is made.
Regressions cover mixed supported/unsupported records, newline continuation and
translated warning rendering without actionable paths.

S6 final validation: 65 snapshot tests passed, 1 Btrfs test ignored in normal
runs. cargo fmt --all -- --check, cargo clippy --workspace --all-targets --
-D warnings, cargo test --workspace, argvus i18n validate and git diff --check
passed. Internal and bilingual user documentation updated. No commits created.

## S7 — shared status and diff execution

Confirmed duplication: compare/status and diff/file_diff maintained separate
command builders and output decoders. Unified status_records provides the parsed
records/count to compare and membership validation; public diff delegates to
file_diff after fresh snapshot and path validation. Diff no longer constructs a
UI comparison with truncation budgeting. Regression initially failed for a
valid path beyond the comparison display prefix, then passed after separation.
UI still only offers actions on its retained paths.

Not confirmed: extra inspections or repeated explicit status calls inside a
single restore_preview. Current code already performs one root inspection, one
status and one path-filtered diff; a recording-runner test now locks down those
counts. Snapper --help confirms status has no per-file operand while diff accepts
files, so status remains necessary for helper membership revalidation. Snapper's
internal traversal cost is not reduced or benchmarked here. No cross-request
cache is introduced; restore_file still revalidates after safety snapshot creation.

S7 final validation: 67 snapshot tests passed, 1 Btrfs test ignored in ordinary
runs. cargo fmt --all -- --check, cargo clippy --workspace --all-targets --
-D warnings, cargo test --workspace and git diff --check passed. Internal
documentation updated. No new UI strings, packaging changes or commits.

## H2 — logind drop-in and reload

Confirmed with failing tests: a missing parent directory prevents drop-in
creation; reload_logind incorrectly returns success when the command fails.
New drop-ins now contain [Login], missing directories are created, updates are
scoped to Login, and reload errors propagate. Legacy sectionless files gain a
Login header. File read and backup errors are not silently discarded. New files
use 0644; existing file permissions are preserved. Tests assert complete output,
section isolation and a safely disabled subprocess reload failure.

Not confirmed as a defect: battery mapping to HandleLidSwitch. systemd defines
HandleLidSwitchExternalPower for AC and HandleLidSwitchDocked for docking;
otherwise it applies HandleLidSwitch. battery is the UI's real caller, while
default and the one-value command are legacy aliases for the fallback. They are
not independent settings. Removing battery would break the existing Power page,
so the aliases are retained and documented. Reference:
https://www.freedesktop.org/software/systemd/man/latest/logind.conf.html
No live logind reload or lid/power event was triggered on the host.

H2 final validation: 273 settings tests passed in the workspace feature set;
cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings,
cargo test --workspace and git diff --check passed. Bilingual Control Center
documentation updated. Real logind reload/hardware behavior not tested to avoid
affecting the active session. No catalog keys, packaging changes or commits.

## H3 — DNS resolver lifecycle

Confirmed: nmcli_failure_does_not_change_resolver failed because resolver bytes
changed despite nmcli failure. Manual/automatic operations now run nmcli before
changing resolver bytes. IPv4 and IPv6 server lists are passed separately, with
an empty list clearing an unused family. Restore rejects resolver/backup symlinks
before command execution and removes the backup only after successful restore.
New manual cycles refresh obsolete backups from the current non-ARGVUS resolver;
repeated manual edits retain the original backup. Missing manual backup fails
closed. Atomic DNS writes use create_new temporary files, check symlinks again
before rename and remove owned temporary files on write failures.

Tests inject paths and command execution, covering nmcli failure, mixed-family
arguments, stale backup renewal, restore failure, successful backup removal,
a second manual cycle, and preservation of a service-managed resolver symlink.
No host DNS/network settings were changed. A filesystem failure after nmcli
success can still leave the profile changed while resolver persistence fails;
that error is propagated, not claimed as an all-or-nothing cross-service transaction.

H3 final validation: all 3 new DNS regressions and the workspace tests passed;
cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings
and git diff --check passed. Bilingual network documentation updated. No host
DNS changes, packaging changes or commits.

## H4 — restricted mirrorlist payload

Confirmed: mirror_apply_rejects_pacman_directives_and_oversized_arguments failed
before the fix because arbitrary pacman directives accompanied a Server line.
A shared core validator now accepts only blank lines, comments and HTTP(S) Server
assignments, requires a nonempty server authority, and limits UTF-8 payload bytes
to 64 KiB. PrivilegedRequest validates before process creation, the helper checks
again, and the direct writer also refuses invalid data before filesystem effects.
Reflector previews use the same validator. Tests cover directives, file/FTP URLs,
empty authorities, malformed lines, pre-elevation/direct-writer rejection and the
exact byte limit. No real mirrorlist was written.

H4 final validation: 278 settings tests passed in the workspace feature set;
cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings,
cargo test --workspace and git diff --check passed. Bilingual Control Center
documentation updated. No new catalog keys, packaging changes or commits.

## H5 — remaining helper hardening

Confirmed by a failing test: write_atomic followed an existing temporary symlink.
Locale, boot, DNS and logind writes now share exclusive create_new/O_NOFOLLOW
staging, reject destination symlinks and remove only their owned temporary on
failure. Existing modes are preserved. chmod EPERM is ignored only after fstatfs
identifies FAT/exFAT; the ESP branch still needs real FAT filesystem validation.
Regression tests cover existing temporary symlinks, destination symlinks, mode
preservation and cleanup after a forced rename failure.

Source confirmation: the UI says remove uninstalled cache packages, so paccache
now receives -r -u -k0; the argument builder has a regression test. Hostnames with
surrounding whitespace are now rejected instead of validating a trimmed value and
executing another value. Empty generated-locale lists are rejected by both the
request validator and executor. Both have validation regressions.

Repository include disable/enable/remove now refuses files with multiple sections,
rather than affecting all sections. A focused include test covers refusal and
single-section acceptance. Include backup is completed before changing pacman.conf,
so backup failure cannot occur after that mutation. Later unlink or key-removal
failures still propagate and are not claimed to form a cross-file transaction.
No real cache, locale, hostname, boot or repository mutation was performed.

H5 final validation: 283 settings tests passed in the focused run; cargo fmt
--all -- --check, cargo clippy --workspace --all-targets -- -D warnings,
cargo test --workspace and git diff --check passed. Bilingual Control Center
documentation updated. FAT/ESP chmod behavior remains a real-filesystem check;
no privileged host mutations or commits were performed.

## C5 — explicit cancellation and retained editor jobs

Confirmed by failing regressions: cancellation set timed_out=true, and closing
the network editor discarded its active job handle. ProcessOutput now includes
cancelled, distinct from timed_out, in buffered and live execution. All recording
runner fixtures across direct consumers were updated. Editor close waits while
an editor/action job exists, preserving the handle and polling its eventual result.
Tests cover cancellation in both runner modes and retained editor completion.

Not confirmed: cancel_refresh_for_action cancelling DNS/action jobs. Current
self.job is assigned only by start_refresh and contains load_page reads; DNS and
other mutations use self.action, which this method does not touch. That existing
separation is preserved. close_editor no longer cancels editor reads either,
because the shared editor handle can also carry a multi-step mutation. Global
application quit remains separately tracked under C10.

C5 final validation: cancellation tests for buffered/live runners and the
editor completion regression passed; cargo fmt --all -- --check, cargo clippy
--workspace --all-targets -- -D warnings, cargo test --workspace and
git diff --check passed. The initial clippy pass exposed one compact services
fixture missing the new field; it was fixed and validation repeated successfully.
Internal and bilingual network documentation updated. No commits created.

## C6 — package load key captured at spawn

Confirmed: stale_package_query_does_not_stamp_the_current_page_key failed because
a result for the prior page replaced packages. Reload jobs now retain their spawn
key (page/query/details target). Finished results or errors are discarded when
that key no longer matches, and a fresh load is started. Matching success stamps
last_loaded with the captured key, not the completion-time key. Special mirror
and repository review jobs are not page-cache loads and do not stamp last_loaded.
Cancellation clears the pending key. Regressions cover changed page, changed
search with a stale error, and a matching successful result. No real package
transaction was performed.

C6 final validation: 94 packages tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. Bilingual Control Center documentation updated.
No new catalog keys, packaging changes or commits.

## C7 — single network action and translated status

Confirmed: wifi_connection_does_not_replace_an_inflight_action failed because
run_wifi replaced the original action status/handle. Wi-Fi now uses the shared
action dispatcher with separate translated progress and success keys. Existing
actions and editor jobs block new mutations; the old result remains polled.
Regression coverage includes retained action/result, editor exclusion and bilingual
progress/success messages with a successful fake task.

Confirmed untranslated success literal Conectado was replaced with the existing
connected translation. Other Portuguese action literals already went through an
action_label translation mapping; their display was not confirmed untranslated.
Those internal literal identifiers and the fallback mapping have been replaced
by explicit translation keys at each call site. Existing en-US/pt-BR catalogs
already contain all required keys; no new keys or real network changes.

C7 final validation: 96 network tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace,
argvus i18n validate and git diff --check passed. Bilingual network documentation
updated. No new catalog keys, packaging changes or commits.

## C8 — bounded, sanitized incremental live output

Confirmed: live_output_limit_is_enforced_after_draining failed before the fix.
run_live now honors per-stream output_limit; absent one it retains at most 8 MiB
per captured stream. Pipes continue draining after the cap, and OutputLimit is
returned after the child finishes. Live display storage is bounded to 10,000
lines / 4 MiB with 64 KiB line fragments, sanitizes before storage, and maintains
an incrementally appended text buffer instead of joining all rows per read.
lines_since exposes only new rows to callers. Package transaction UI translates
and counts only newly appended rows, caches the text and wrapped-line count,
and shows a bilingual notice when display data is omitted. Rendering still
processes the bounded cached text through ratatui; no full-output translation or
join is performed per tick. Capture overflow is an error, not a claim that the
underlying transaction failed or was rolled back.

Tests cover live capture overflow, terminal sequence sanitization, display line
caps, incremental cursors/cache and a single translated truncation notice.
The existing invalid-UTF-8/1 MiB draining regressions remain passing. No real
package transaction was run.

C8 final validation: 56 core tests, 2 process isolation tests and 96 packages
tests passed. cargo fmt --all -- --check, cargo clippy --workspace --all-targets
-- -D warnings, cargo test --workspace, argvus i18n validate and git diff --check
passed. Full tests were repeated after the final byte-boundary separator guard.
Bilingual Control Center documentation and truncation catalog key updated.
No real package transaction, packaging change or commit was performed.

## C9 — terminal sanitization state machine

Confirmed with four failing regressions: nonalphabetic CSI finals consumed text,
DCS/APC/PM/SOS payloads remained visible, unterminated OSC swallowed following
lines, and bidi/invisible format controls survived. terminal_text now uses explicit
ground/escape/intermediate/CSI/control-string/string-escape states. CSI ends on
any ASCII 0x40–0x7E final. OSC recognizes BEL/ST; other strings recognize ST.
C1 Unicode forms and charset selection escapes are handled too. Unterminated OSC
strings drop data through newline/EOF, then resume ordinary text; other strings
retain multiline payload suppression until ST or EOF. Payload on the
same unterminated line cannot safely be distinguished from visible text.

Filtering removes U+200B–200F, U+202A–202E, U+2066–206F, U+FEFF, U+061C and U+2060;
ordinary Unicode text, newline and tab remain. Added regressions cover every
reported class, C1 forms, Unicode preservation and idempotence. No user-facing
strings or runtime configuration changes were introduced. This sanitizer accepts
whole strings; arbitrary sequences split across independent calls are not a
persistent terminal stream parser.

C9 final validation: 62 core tests and 2 process isolation tests passed;
cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings,
cargo test --workspace and git diff --check passed. Internal sanitizer behavior
documented. No new catalog keys, packaging changes or commits.

## P4 — Arch pkgbase and mkinitcpio arrays

Confirmed by two failing regressions: kernel package equaled the release directory
instead of linux-lts, and MODULES tokens retained parentheses. Detection now reads
/usr/lib/modules/<release>/pkgbase for image/initramfs/fallback/preset naming,
while version and current matching retain the release. Missing/invalid pkgbase
falls back to the release name; pkgbase accepts only safe filename characters.
mkinitcpio literal arrays support parentheses, multiline values, comments, quotes
and escaped characters without executing shell substitutions or sourcing config.
Regression tests cover real filename shapes, version/current flags and multiline
quoted literal arrays. Arbitrary shell expressions in mkinitcpio are not evaluated.

Read-only host integration test real_arch_kernel_and_mkinitcpio_inventory passed
against the installed release and pkgbase and /etc/mkinitcpio.conf. Reproduce with:

```sh
cargo test -p argvus-control-center-boot real_arch_kernel_and_mkinitcpio_inventory -- --ignored
```

No boot configuration or images were written.

P4 final validation: 35 boot tests passed, plus the explicitly run read-only
Arch integration test (ignored in ordinary runs). cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. Bilingual Control Center documentation updated.
No new catalog keys, packaging changes or commits.

## P5 — power read fallback and apply-before-persist

Confirmed with failing regressions: a missing drop-in returned None despite a
main-config value, and rejected privileged lid changes persisted the requested
canonical value first. logind reads now fall back to the main file when the
drop-in is missing or lacks the key. Lid and power-button mutations apply through
the privileged helper before persisting their canonical setting. Keep-awake
persists only after a successful non-cancelled, non-timeout runtime command.

Injected application/persistence callbacks and a recording runner cover missing
drop-in, rejected authorization, successful ordering and keep-awake failures.
If persistence fails after a successful runtime application, that error still
propagates; this change does not claim rollback of a completed system action.
The existing keep-awake script invocation/path is preserved; no shell-to-Rust
migration or live power action was performed. P5 does not change precedence of
valid canonical values over logind fallback values.

P5 final validation: 41 power tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. Bilingual Control Center documentation updated.
No new catalog keys, packaging changes, live session mutations or commits.

## P6 — Bluetooth removal identity, battery and detail queries

Confirmed with failing regressions: removal confirmation stored no device address,
Battery Percentage: 0x55 (85) parsed as None, and info query concurrency was 1.
Confirmation now owns the original address and dispatches it directly, independent
of reordered/disappearing scan rows. Cancellation drops the captured address.
Battery parsing accepts matching hexadecimal/decimal output, hexadecimal alone,
decimal and legacy brackets, enforcing 0..100 and rejecting inconsistent pairs.

Read-only info requests now run in batches of at most four scoped workers, with
5-second per-info timeouts; ordinary Bluetooth actions retain their existing
20-second timeout. Recording-runner tests cover eight devices, concurrency bounds,
info timeout and battery results. UI regression confirms the original target even
after inventory replacement/removal. No real device pairing/removal was attempted.
Queries still scale by batches for very large inventories; no new daemon/cache.

P6 final validation: 16 Bluetooth tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. Bilingual network documentation updated. No new
catalog keys, packaging changes, physical-device mutations or commits.

## S8 — mount paths decoded as Unix bytes

Confirmed with failing regressions: /m\303\263dulo decoded as /mÃ³dulo, and an
escaped Unicode mountpoint failed to block restoration underneath it. Octal
escapes now produce raw bytes and OsString::from_vec builds the PathBuf without
Latin-1 conversion. mountinfo is read/split as bytes so literal non-UTF-8 paths
also remain representable. Invalid octal escapes, byte overflow and NUL paths
are rejected. Regressions cover UTF-8, spaces, escaped/literal non-UTF-8 bytes,
malformed escapes and the actual restore boundary predicate. No mounts or
privileged runtime files were changed.

S8 final validation: 70 snapshot tests passed, 1 Btrfs integration test ignored
in ordinary runs. cargo fmt --all -- --check, cargo clippy --workspace
--all-targets -- -D warnings, cargo test --workspace and git diff --check passed.
Internal snapshot documentation updated. No new catalog keys, packaging changes
or commits.

## S9 — lock-specific contention errors

Confirmed: package_lock_contention_is_not_reported_as_another_restore failed with
Invalid: another selective restore is in progress. Lock acquisition now receives
an explicit purpose and returns PackageUpdateBusy or RestoreBusy for contention.
Other flock failures preserve their OS error instead of being mislabeled busy.
The UI renders separate en-US/pt-BR messages without displaying the raw diagnostic
as the user explanation. Both sides share the updated ErrorKind wire enum.
Tests hold each lock, verify no Snapper command starts, check package lock release
after restore-lock refusal, and round-trip/render both errors in both languages.
No runtime system locks were acquired; all lock files were in test directories.

S9 final validation: 73 snapshot tests passed, 1 Btrfs test ignored in ordinary
runs. cargo fmt --all -- --check, cargo clippy --workspace --all-targets --
-D warnings, cargo test --workspace, argvus i18n validate and git diff --check
passed. Internal snapshot documentation and bilingual lock messages updated.
No packaging changes or commits.

## S10 — retention configuration arguments and provenance

Confirmed by a failing regression: multiple settings were passed as one Snapper
argv value. set_retention now emits one KEY=VALUE argument per pair and retains
read-back checks. UI/helper wire configdata remains one validated space-delimited
string. Upstream client/snapper/cmd-set-config.cc passes get_args to read_configdata;
client/misc.cc splits each argv item at its first '=' (it does not split spaces).
The installed manpage's quoted example is misleading for this parser. Sources:
https://github.com/openSUSE/snapper/blob/master/client/snapper/cmd-set-config.cc
https://github.com/openSUSE/snapper/blob/master/client/misc.cc

Removed the unreachable unknown-key branch and duplicate editable-key table;
retention defaults define the whitelist. The shared validated configdata parser
is now also used by the settings helper, replacing parse_changes. The unused old
parameter in save_retention was removed; confirmation still retains old/new text.
Tests cover separate argv entries, shared-parser validation and bilingual fallback
labels. The existing single-key argument test still passes.

Runtime fallback values checked against the installed snapper-configs(5) manual
are consistent (e.g. cleanup=no and minimum age=3600); differing values in
/usr/share/snapper/config-templates/default are creation-template choices, not
proof of a runtime-default bug. No dynamic template sourcing was added. UI labels
now explicitly say ARGVUS fallback / setting absent, rather than claiming a
value was obtained from the installed Snapper template. The legacy serialized
SettingOrigin::SnapperDefault name is retained for compatibility and documented.
No real Snapper configuration was changed.

S10 final validation: 76 snapshot tests passed, 1 Btrfs test ignored in normal
runs. cargo fmt --all -- --check, cargo clippy --workspace --all-targets --
-D warnings, cargo test --workspace, argvus i18n validate and git diff --check
passed. Internal and bilingual user documentation plus fallback catalogs updated.
No real Snapper mutation, packaging change or commit was performed.

## S11 — Snapshot UI state and row caching

Confirmed before correction with failing regressions: entering ChooseConfig kept
a previous snapshot index; failed retention writes retained stale values; repeated
row requests rebuilt the entire vector. Configuration choice now resets selection
and previous configuration state. A failed retention mutation clears stale values
and rereads retention for the selected configuration, preserving the error notice.
Rows and styled lines are cached until input, completed jobs, reload, theme or
language changes. Idle polls reuse the cache. Scroll bounds already implemented
for S4 are retained; unchanged frames no longer rescan the content for bounds.
Tests cover the three confirmed failures, completed-job invalidation, public
language changes, and successful retention read-back after a failed write.
No new visible strings or protocol changes were required.

S11 final validation: 81 snapshot tests passed, 1 real-Btrfs test ignored in
normal runs. cargo fmt --all -- --check, cargo clippy --workspace --all-targets
-- -D warnings, cargo test --workspace and git diff --check passed. Internal
and bilingual user documentation updated. No catalog or packaging changes,
real-system mutation or commit was performed.

## S12 — Pre-upgrade snapshots with multiple root configurations

Confirmed with a failing fake-runner regression: create_pre_upgrade returned
Incompatible and requested an unavailable selection. Automatic creation now
prefers the verified root configuration named root, otherwise the first verified
name alphabetically. The existing mutation path revalidates membership before
creating; manual UI selection remains explicit. No persistent selection, helper
protocol or package transaction changes were introduced. Regression covers both
selection rules and number cleanup. Bilingual and internal docs updated, including
stale pre-update cleanup text discovered while editing the same section.
No real package update or Snapper mutation was performed.

S12 final validation: 82 snapshot tests passed, 1 real-Btrfs test ignored in
normal runs. cargo fmt --all -- --check, cargo clippy --workspace --all-targets
-- -D warnings, cargo test --workspace and git diff --check passed. No new
visible strings, catalogs, packaging changes or commits. Real Snapper/Polkit
pre-upgrade integration remains unexecuted; verification used the fake runner.

## H6 — Serialize package mutations with selective restore

Confirmed with a failing regression: while the shared flock was held, install
reached the pacman executor. The package dispatcher now acquires the same
/run/lock/argvus-package-update-snapshot.lock for every package operation,
including install, removal variants, reinstall, downgrade, refresh, cache,
mirror and repository changes. The RAII guard remains alive through the entire
operation; upgrades no longer reacquire it internally. Snapshot protection is
still required only for the previously covered upgrade operations.
Tests cover refusal before dispatch across twelve actions, release on dispatch
error, and the existing acquisition/drop/reacquisition test. No real package
transaction or system lock was used; tests use isolated temporary lock files.

H6 final validation: cargo fmt --all -- --check, cargo clippy --workspace
--all-targets -- -D warnings, cargo test --workspace and git diff --check passed.
Both new H6 tests passed in the workspace's 286 settings tests. The standalone
settings suite also passed before the additional release-on-error test was added.
Internal and bilingual user documentation updated; no new visible strings,
protocol, catalogs or packaging changes. No commit was performed.

## C10 — Retain the application during pending mutations

Confirmed before correction by an event regression: q set quit=true while a
Bluetooth action was pending on another page. Both q and Ctrl+C now consult
all compiled page action slots before requesting exit. Pending package actions,
network/editor jobs, snapshot mutations, settings/admin/input tasks, configuration
writes and other page actions block exit without cancelling their handles.
A translated footer asks the user to wait; completion keeps the application open
so the result can be reviewed, then the user can request exit again. Read-only
refresh slots do not block exit. A few mixed read/write slots (network editor,
settings admin/DND) conservatively block until their result is consumed. Pending
display revert decisions also block exit.
Tests cover both quit keys across routes, exit after result processing, ordinary
refresh, pending package/network/editor jobs, and bilingual notice rendering.
This governs keyboard exit in the raw terminal UI; forced external termination
and terminal/session loss are outside this guarantee. No privileged operation
was executed for validation.

C10 final validation: 83 Control Center tests passed, including three new event
and rendering tests. Two additional package/network pending-state regressions
passed in cargo test --workspace. cargo fmt --all -- --check, cargo clippy
--workspace --all-targets -- -D warnings, cargo test --workspace, argvus i18n
validate and git diff --check passed. Added control_center.quit_operation_pending
in en-US and pt-BR; bilingual rendering verified. Internal and bilingual user
documentation updated. No protocol, packaging or commit changes. Real privileged
transactions and externally forced termination were not exercised.

## C11 — Network text input and shared editor columns

Failing regressions confirmed Ctrl+W inserting w in simple network fields,
missing paste, Ctrl+Alt characters being discarded (or saving on s), vertical
movement using character counts, and horizontal scroll leaving the cursor over
a wide glyph rather than after the final x. Network password, DNS and search
now retain modifiers and accept bracketed paste with controls/line breaks
removed; unsupported Ctrl-only/Alt-only character shortcuts are ignored.
The shared text_character policy accepts printable unmodified/Shift text and
Ctrl+Alt text, preserving AltGr encodings; Ctrl+S still saves without Alt.
Vertical positioning uses displayed grapheme widths and maps back to character
indices, accounting for hidden tabs and masked values. Horizontal scroll aligns
to a grapheme boundary on the cursor line. Ratatui 0.30.2 itself scrolls in
cells: a general character-vs-cell unit mismatch was not confirmed; the actual
confirmed horizontal defect occurs when the requested cell offset cuts a wide
grapheme retained by Paragraph.
The terminal protocol cannot distinguish an AltGr character encoded as Ctrl+Alt
from the same explicit Ctrl+Alt chord; printable events use text semantics.
Tests use synthetic KeyEvents and Ratatui TestBackend; physical pt-BR AltGr in
a real terminal remains a manual integration check. No new visible strings,
helper protocol or package dependencies are introduced.

An additional failing event test confirmed that the global Ctrl+C shortcut
also captured Ctrl+Alt+c. Exit now requires Ctrl without Alt, preserving the
shared AltGr text policy before page dispatch. For manual validation, run
`localectl status` to inspect the layout, then `argvus control-center network`
in the target terminal. Open a text field, type AltGr characters, paste text,
and cancel without applying; verify Ctrl+S separately in the profile editor.

C11 final validation: 98 network tests and 92 TUI tests passed in focused
runs. All five new C11 regressions passed in cargo test --workspace, including
the global AltGr/Ctrl+C test. cargo fmt --all -- --check, cargo clippy
--workspace --all-targets -- -D warnings, cargo test --workspace and
git diff --check passed. Bilingual Control Center/network user docs and
internal UX notes updated. No catalogs or packaging changes were needed;
no commit was performed. Physical terminal/keyboard AltGr remains unverified.

## P7 — Audio preferences belong to the default output only

Two failing regressions confirmed that microphone volume/mute persisted output
preferences, and an observed 150% volume was clamped to 100%. Canonical config
and its projection only define the default sink (0–100%), not source settings.
The backend now verifies the current @DEFAULT_AUDIO_SINK@ identity before
persisting a successful volume/mute mutation. Inputs, other sinks and unknown
default identity leave canonical output preferences unchanged. Runtime changes
still target the selected node. No input config keys or independent persistent
state were invented.
Observed AudioDevice.volume is widened to u32 to preserve amplified readings
(including >255%). Non-finite, negative and unrepresentable readings become
unknown, not a misleading clamped value. UI draft edits and canonical defaults
remain bounded to 0–100%. Draft comparison and stepping handle the wider
observation without integer overflow or an implicit change on refresh.
Tests cover input isolation, default/nondefault output persistence, unknown
default identity, amplified/invalid readings, and an explicit bounded UI edit.
No live audio levels, microphone settings or canonical user files were changed.

P7 validation checkpoint (stopped as requested on workspace failure):
- cargo fmt --all -- --check: passed.
- cargo clippy --workspace --all-targets -- -D warnings: passed.
- cargo test --workspace: failed in existing argvus-tui test
  icon_scale::tests::test_parse_scale_ratio_custom at icon_scale.rs:313.
  Observed (3, 4), expected (2, 3); TUI result 91 passed, 1 failed.
  Both scale-ratio tests mutate ARGVUS_ICON_SCALE without synchronization,
  consistent with interference between parallel tests. No causality from the
  audio changes has been established, and no unrelated test fix was applied.
- All 20 audio tests passed in that workspace run, including five P7 tests.
- git diff --check: passed.
- Read-only real-system wpctl inspect @DEFAULT_AUDIO_SINK@ succeeded and
  returned a numeric id with media.class=Audio/Sink; no settings were changed.
At this checkpoint P7 was not declared complete, and P8 was not started.
The failure details above are preserved; temporary logs were removed after the
successful final validation below. Bilingual user docs and internal UX notes
reflect the implemented behavior. No new visible strings, catalogs, helper
protocol, dependencies, packaging changes or commits.

P7 completed after the user's request to resolve the validation blocker. The
icon-scale parser now accepts an explicit optional value internally; production
still reads ARGVUS_ICON_SCALE, while tests no longer set/remove the process-wide
variable. This removes the confirmed unsynchronized default/custom test
interference without changing runtime scaling rules. A fallback/valid-input
regression covers missing and malformed ratios.

Final P7 validation: 20 audio tests and four icon-scale tests passed in focused
runs. cargo fmt --all -- --check, cargo clippy --workspace --all-targets --
-D warnings, cargo test --workspace and git diff --check all passed. Five P7
audio regressions and the icon-scale regression passed in the complete suite.
Bilingual Control Center user docs and internal UX notes are synchronized.
No live audio mutation, catalog, dependency, packaging or helper-protocol
changes, and no commits. P8–P10 remain pending; P8 was not started.

## P8 — PCI GPU names and graphics probe scope

Two failing fake-runner regressions confirmed that splitting lspci at the first
colon retained part of the PCI address/class, and global renderer strings were
copied onto both Intel and NVIDIA records. A shared parser now separates the
address token from the class/model delimiter, handles domain-qualified and short
addresses, retains human model brackets, and removes only numeric PCI ID tags.
Both sysfs-backed lookup and lspci fallback use it. Targeted lookups request
-D and accept only the matching PCI address; empty sysfs identities no longer
issue an unscoped lspci lookup.
Global glxinfo -B and vulkaninfo --summary results now live in HardwareSnapshot
graphics metadata, displayed in Hardware Overview > Summary. They are not
attributed to physical cards without a verified identity. Vulkan deviceName
records accept the real equals separator and retain all reported names, including
software devices, rather than selecting the first one for every card. Unknown
per-device API fields remain unset. The global probes have 10-second deadlines,
1 MiB output limits and reject failed/timed-out/cancelled results.
Six regressions cover PCI parsing and model brackets, multiple GPUs, global
Vulkan enumeration, failed probes and bilingual Summary/per-GPU rendering.
Two translated labels were added in en-US and pt-BR. No route, helper protocol,
dependency or packaging changes. Per-physical-device OpenGL/Vulkan capability
attribution remains intentionally unavailable without a reliable device key.

P8 final validation: 15 hardware tests passed, including six new regressions.
cargo fmt --all -- --check, cargo clippy --workspace --all-targets --
-D warnings, cargo test --workspace, argvus i18n validate and git diff --check
passed. Read-only `lspci -D -nn` on the host confirmed the tested format with
a domain-qualified NVIDIA address and a commercial name in brackets. No GPU
settings were changed. Multi-GPU attribution was exercised with fake output,
not a real multi-GPU graphics session. Bilingual user docs/internal UX notes
and en-US/pt-BR catalogs updated. No commits were performed.

## P9 — Clear greetd autologin before locking or deleting its account

The active implementation is argvus-accounts-core/src/admin.rs (the settings UI
sends its structured account request to this provider). Two failing regressions
confirmed that account restriction reached the shadow command with initial_session
still present and ignored a simulated autologin-cleanup failure. Lock(true) and
delete now validate account protections, remove a matching greetd initial_session
atomically, then invoke usermod/userdel. Unlock does not alter autologin.
A missing greetd config or a different autologin user causes no write. Read/write
errors propagate before account mutation. If the subsequent shadow command fails,
autologin stays disabled; it is not automatically restored. No existing session
is terminated and greetd is not restarted; the file affects future starts.
The existing autologin section transform and temporary-file replacement are
reused, with an injected path for isolated tests. File mode and other sections
are preserved. Six new tests cover lock/delete ordering, protection ordering,
cleanup failure, shadow failure, missing/other-user configs and file preservation.
No real account, /etc/greetd file or service was modified.

P9 final validation: 57 account-core unit tests, 12 integration tests and one
doctest passed in the focused run. All six P9 regressions passed in the complete
workspace suite. cargo fmt --all -- --check, cargo clippy --workspace
--all-targets -- -D warnings, cargo test --workspace and git diff --check passed.
Bilingual Accounts and Control Center user docs and internal Accounts notes
updated. No new visible strings/catalogs, helper protocol, dependencies or
packaging changes. Real login/lock/delete operations were not exercised;
validation used temporary greetd files and simulated account commands.
No commits were performed.

## P10 — Authorize handoff clients and bind preparation to DRM identity

Two failing regressions confirmed that any connecting UID could request a
release/theme change and that a surface prepared on one observer descriptor
could be passed to a different acquired descriptor. The socket must serve the
greeter and a different authenticated session UID: its connect permissions
remain 0666, but every command is now authorized with Linux SO_PEERCRED before
reading any payload. Only root, the helper owner and the authenticated account
UID are accepted, including while holding the overlay. The parent directory
remains greeter-owned 0755; other users cannot replace the socket. No claim is
made that filesystem mode alone isolates access.
The login UI carries the selected account UID through PAM prompt continuations.
Only after greetd authenticates does the greeter start the helper with its
internal --session-uid argument. No UI/session client command format changes
are required. Omitting that argument authorizes only root and the helper owner.
Commands are capped at 4096 bytes and retain read deadlines; negative
acknowledgements are no longer reported as successful requests.
Prepared surfaces carry their original fd. Master acquisition waits on that
observer when one was prepared, instead of selecting an idle unrelated GPU or
reopening a different file description. A final fd check discards mismatched
preparation before rendering. Timing marks include fd values for verification.
Seven new regressions cover unrelated-peer refusal with real SO_PEERCRED, UID
policy, real selected-UID acceptance, bounded payloads, refused replies, fd
mismatch, and UID preservation through prompts. No live DRM or greetd service
was changed. Real multi-GPU/VT behavior remains unverified.

Manual verification in a disposable multi-GPU session (not executed here):
1. `stat -c '%a %U %G %n' /run/argvus-greeter /run/argvus-greeter/handoff.sock`
   confirms greeter directory ownership and the deliberately shared socket.
2. During an authenticated transition,
   `sudo -u audit-other /usr/lib/argvus/greeter/handoff --send ping` must fail
   for an unrelated test account; the authenticated account's ping must succeed.
   Do not send release to an active production transition as a test.
3. After login, `rg 'handoff.prepared|handoff.master-acquired|handoff.page-flip|handoff.retired|handoff.replacement-timeout' /run/argvus-greeter/timing.log`
   must show identical prepared/acquired fd values and a completed transition.
4. Repeat with display connectors on each physical GPU and with one GPU idle;
   confirm no cross-device framebuffer errors and that the user compositor
   acquires DRM after release. No real-hardware result is claimed.

P10 final validation: 33 greeter tests and 19 handoff tests passed in focused
runs, including seven new regressions. cargo fmt --all -- --check, cargo clippy
--workspace --all-targets -- -D warnings, cargo test --workspace,
cargo build --profile greeter -p argvus-greeter and git diff --check passed.
The final workspace run followed the last source adjustment. Bilingual Greeter
user pages and greeter/lock/splash developer pages updated. No new authored
user-visible messages or catalogs, dependencies, installed paths, packaging
changes or commits. Session scripts retain their existing command protocol.
Real login/VT/multi-GPU display behavior is not claimed: verification is limited
to real local peer-credential socket tests, pure fd-identity checks and compile/
workspace validation. The manual commands above remain required in a dedicated
session before claiming hardware validation.
