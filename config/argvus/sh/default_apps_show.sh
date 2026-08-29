#!/usr/bin/env sh
# shellcheck shell=sh
#
# default_apps_show.sh — opens the argvus-default-apps graphical selector.
# Resolves the binary by PATH (or /usr/bin) so the sidebar card and shortcuts
# can launch it regardless of the current $PATH of the calling process.

set -eu

BIN="argvus-default-apps"

# Resolve the tool; prefer the user-local build (which may be newer than the
# system package) and fall back to PATH so hyprland's minimal env still works.
FOUND=""
for p in "$HOME/.local/bin/$BIN" /usr/local/bin/$BIN /usr/bin/$BIN; do
  if [ -x "$p" ]; then
    FOUND="$p"
    break
  fi
done
if [ -z "$FOUND" ]; then
  FOUND="$(command -v "$BIN" 2>/dev/null || true)"
fi

if [ -z "$FOUND" ]; then
  notify-send "Default Programs" "argvus-default-apps not installed" >/dev/null 2>&1 || true
  exit 1
fi

# Avoid clutter for the persistent "show again" refresh; just launch.
"$FOUND" show
