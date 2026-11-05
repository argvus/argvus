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

S2–S12, H1–H6, C1–C11 and P1–P10 remain pending confirmation and correction.

S1 validation: 52 snapshot tests passed; cargo fmt --all -- --check,
cargo clippy --workspace --all-targets -- -D warnings, cargo test --workspace
and git diff --check passed. Real Btrfs validation remains pending.
