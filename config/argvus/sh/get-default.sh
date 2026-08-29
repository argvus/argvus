#!/usr/bin/env sh
# shellcheck shell=sh
#
# get-default.sh <category> — prints the configured default app (binary) for a
# category, falling back to the Argvus built-in default when no state exists.
#
# Reads the single `defaults.json` written by `argvus-default-apps`, with the
# same path precedence used across Argvus:
#   $XDG_CONFIG_HOME/argvus/defaults.json
#       -> $XDG_STATE_HOME/argvus/defaults.json   (written by the Rust tool)
#       -> /usr/share/argvus/defaults.json
#
# Categories match the argvus-default-apps keys:
#   terminal, file_manager, text_editor, terminal_editor, browser,
#   image_viewer, pdf_viewer, video_player, audio_player, archive, launcher

set -eu

CATEGORY="${1:-}"
if [ -z "$CATEGORY" ]; then
  printf 'Usage: get-default.sh <category>\n' >&2
  exit 2
fi

XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"

# Locate the defaults state file by precedence.
find_state_file() {
  _user="$XDG_CONFIG_HOME/argvus/defaults.json"
  _state="$XDG_STATE_HOME/argvus/defaults.json"
  _system="/usr/share/argvus/defaults.json"
  if [ -f "$_user" ]; then
    printf '%s\n' "$_user"
  elif [ -f "$_state" ]; then
    printf '%s\n' "$_state"
  elif [ -f "$_system" ]; then
    printf '%s\n' "$_system"
  fi
}

fallback() {
  case "$1" in
    terminal) printf 'kitty\n' ;;
    file_manager) printf 'spf\n' ;;
    text_editor) printf 'mousepad\n' ;;
    terminal_editor) printf 'vim\n' ;;
    browser) printf 'xdg-open\n' ;;
    image_viewer) printf 'imv\n' ;;
    pdf_viewer) printf 'zathura\n' ;;
    video_player) printf 'mpv\n' ;;
    audio_player) printf 'audacious\n' ;;
    archive) printf 'xarchiver\n' ;;
    launcher) printf 'rofi\n' ;;
    *) printf '\n' ;;
  esac
}

STATE_FILE="$(find_state_file)"

if [ -n "$STATE_FILE" ]; then
  if command -v jq >/dev/null 2>&1; then
    val="$(jq -r --arg k "$CATEGORY" '.[$k] // empty' "$STATE_FILE" 2>/dev/null || true)"
    if [ -n "$val" ] && [ "$val" != "null" ]; then
      printf '%s\n' "$val"
      exit 0
    fi
  else
    # Minimal fallback without jq: grep the first value for "key".
    val="$(sed -n "s/^[[:space:]]*\"${CATEGORY}\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p" "$STATE_FILE" | head -1)"
    if [ -n "$val" ]; then
      printf '%s\n' "$val"
      exit 0
    fi
  fi
fi

fallback "$CATEGORY"
