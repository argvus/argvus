#!/usr/bin/env sh

set -eu

# Opens the argvus-storage context menu anchored to the pointer position at the
# moment this script runs (i.e. at the click/keypress), so the menu does not
# end up following the mouse while the GTK popup is still initializing.
# The pinned coordinates are only honored in gui mode; rofi mode positions
# itself and ignores them.

pos="$(hyprctl cursorpos -j 2>/dev/null || true)"
x="$(printf '%s' "$pos" | sed -n 's/.*"x": *\(-\{0,1\}[0-9][0-9]*\).*/\1/p' || true)"
y="$(printf '%s' "$pos" | sed -n 's/.*"y": *\(-\{0,1\}[0-9][0-9]*\).*/\1/p' || true)"

if [ -n "$x" ] && [ -n "$y" ]; then
  exec argvus-storage menu --x "$x" --y "$y"
fi

exec argvus-storage menu
