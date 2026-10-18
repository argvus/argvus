#!/usr/bin/env sh

set -eu

if [ "${1:-}" = "--surface" ] && [ "$#" -ge 3 ] && command -v jq >/dev/null 2>&1; then
  surface_x="$(printf '%s\n' "$2" | sed -n '/^-\{0,1\}[0-9][0-9]*$/p')"
  surface_y="$(printf '%s\n' "$3" | sed -n '/^-\{0,1\}[0-9][0-9]*$/p')"
  origin="$(
    hyprctl layers -j 2>/dev/null |
      jq -r --arg output "${WAYBAR_OUTPUT_NAME:-}" '
        [
          to_entries[]
          | select($output == "" or .key == $output)
          | .value.levels
          | to_entries[].value[]
          | select(.namespace == "waybar" and .w > (.h * 4))
          | [.x, .y, .h]
        ]
        | first // empty
        | @tsv
      ' 2>/dev/null || true
  )"
  origin_x="$(printf '%s\n' "$origin" | cut -f1)"
  origin_y="$(printf '%s\n' "$origin" | cut -f2)"
  bar_height="$(printf '%s\n' "$origin" | cut -f3)"
  origin_x="$(printf '%s\n' "$origin_x" | sed -n '/^-\{0,1\}[0-9][0-9]*$/p')"
  origin_y="$(printf '%s\n' "$origin_y" | sed -n '/^-\{0,1\}[0-9][0-9]*$/p')"
  bar_height="$(printf '%s\n' "$bar_height" | sed -n '/^-\{0,1\}[0-9][0-9]*$/p')"
  if [ -n "$surface_x" ] && [ -n "$surface_y" ] && [ -n "$origin_x" ] && [ -n "$origin_y" ] && [ -n "$bar_height" ]; then
    exec argvus removable-devices menu \
      --x "$((origin_x + surface_x))" \
      --y "$((origin_y + surface_y))" \
      --bar-bottom "$((origin_y + bar_height))"
  fi
fi

if [ "${1:-}" = "--root" ] && [ "$#" -ge 3 ]; then
  root_x="$(printf '%s\n' "$2" | sed -n '/^-\{0,1\}[0-9][0-9]*$/p')"
  root_y="$(printf '%s\n' "$3" | sed -n '/^-\{0,1\}[0-9][0-9]*$/p')"
  if [ -n "$root_x" ] && [ -n "$root_y" ]; then
    exec argvus removable-devices menu --x "$root_x" --y "$root_y"
  fi
fi

# Compatibility fallback for Waybar versions without event-coordinate support.
# The pinned coordinates are only honored in gui mode; rofi mode positions
# itself and ignores them.

pos="$(hyprctl cursorpos -j 2>/dev/null || true)"
x="$(printf '%s' "$pos" | sed -n 's/.*"x": *\(-\{0,1\}[0-9][0-9]*\).*/\1/p' || true)"
y="$(printf '%s' "$pos" | sed -n 's/.*"y": *\(-\{0,1\}[0-9][0-9]*\).*/\1/p' || true)"

if [ -n "$x" ] && [ -n "$y" ]; then
  exec argvus removable-devices menu --x "$x" --y "$y"
fi

exec argvus removable-devices menu
