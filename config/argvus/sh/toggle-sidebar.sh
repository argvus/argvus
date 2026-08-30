#!/usr/bin/env sh
# shellcheck shell=sh
#
# toggle-sidebar.sh — toggles the sidebar-right on/off.
#
# The sidebar runs as a long-lived quickshell process (qs -c sidebar-right).
# The bare `qs ... ipc call sidebar toggle` only works while that process is
# already running; on a fresh user/session it may not be up yet, so the button
# and SUPER+, would silently do nothing. This helper starts the sidebar if it
# is not running, otherwise it toggles it via IPC.

set -eu

CONFIG="sidebar-right"

# Is the sidebar process already up?
if pgrep -f "qs -c $CONFIG" >/dev/null 2>&1; then
  qs -c "$CONFIG" ipc call sidebar toggle >/dev/null 2>&1 || {
    # Race: process died between check and call — start it instead.
    nohup qs -c "$CONFIG" >> /tmp/quickshell-sidebar.log 2>&1 &
  }
else
  nohup qs -c "$CONFIG" >> /tmp/quickshell-sidebar.log 2>&1 &
fi

exit 0
