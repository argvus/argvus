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

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
# shellcheck disable=SC1091
. "$ARGVUS_BOOTSTRAP"

CONFIG="argvus-control-panel"
QS_LOG="$(paths_cache quickshell)/argvus-control-panel.log"
mkdir -p "${QS_LOG%/*}"

# Is the sidebar process already up?
if pgrep -f "qs -c $CONFIG" >/dev/null 2>&1; then
  qs -c "$CONFIG" ipc call sidebar toggle >/dev/null 2>&1 || {
    # Race: process died between check and call — start it instead.
    nohup qs -c "$CONFIG" >> "$QS_LOG" 2>&1 &
  }
else
  nohup qs -c "$CONFIG" >> "$QS_LOG" 2>&1 &
fi

exit 0
