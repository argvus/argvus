#!/usr/bin/env sh
# shellcheck shell=sh
#
# toggle-sidebar.sh — toggles the argvus-control-panel on/off.
#
# The sidebar runs as a long-lived quickshell process (qs -c argvus-control-panel).
# The bare `qs ... ipc call sidebar toggle` only works while that process is
# already running; on a fresh user/session it may not be up yet, so the button
# and SUPER+, would silently do nothing. This helper starts the sidebar if it
# is not running, otherwise it toggles it via IPC.

set -eu

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
# shellcheck disable=SC1091
. "$ARGVUS_BOOTSTRAP"

CONFIG="argvus-control-panel"

if qs -c "$CONFIG" ipc call sidebar toggle >/dev/null 2>&1; then
  exit 0
fi

command -v systemctl >/dev/null 2>&1 || exit 0
systemctl --user start argvus-shell.service >/dev/null 2>&1 || exit 0
sleep 0.3
qs -c "$CONFIG" ipc call sidebar toggle >/dev/null 2>&1 || true

exit 0
