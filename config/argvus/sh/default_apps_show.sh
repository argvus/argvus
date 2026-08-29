#!/usr/bin/env sh
# shellcheck shell=sh
#
# default_apps_show.sh — opens the argvus-default-apps graphical selector.
# Resolves the binary by PATH (or /usr/bin) so the sidebar card and shortcuts
# can launch it regardless of the current $PATH of the calling process.

set -eu

BIN="argvus-default-apps"

# Resolve the tool; prefer PATH, fall back to common locations.
FOUND="$(command -v "$BIN" 2>/dev/null || true)"
if [ -z "$FOUND" ]; then
  for p in /usr/bin/$BIN /usr/local/bin/$BIN "$HOME/.local/bin/$BIN"; do
    if [ -x "$p" ]; then
      FOUND="$p"
      break
    fi
  done
fi

if [ -z "$FOUND" ]; then
  notify-send "Default Programs" "argvus-default-apps not installed" >/dev/null 2>&1 || true
  exit 1
fi

# Avoid clutter for the persistent "show again" refresh; just launch.
"$FOUND" show
