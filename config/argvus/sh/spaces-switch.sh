#!/usr/bin/env sh
# spaces-switch - customize window gaps and waybar spacing.
# Usage: spaces-switch.sh [--status|--defaults|--get <key>|--set <key> <value>|--reset [key]|--apply-static|--apply]
# Keys: gaps_in, gaps_out, waybar (margin for both bars).
# Values are clamped to a minimum of the active theme's defaults.
# shellcheck disable=SC1091

set -u

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"
ARGVUS_MUTABLE_CONFIG=1

STATE_DIR="${ARGVUS_CONFIG_HOME}/argvus"
SPACES_FILE="${STATE_DIR}/.spaces"
ACTIVE_FILE="${STATE_DIR}/.active-theme"
WAYBAR_CFG="$(paths_config waybar/config.jsonc)"
WAYBAR_SYSINFO="$(paths_config waybar/sysinfo.jsonc)"
THEMES_DIR="$(paths_config hypr/themes)"
DEFAULT_THEME="argvus-dark-aether"
MAX_VALUE=100

is_float_theme() {
  _theme="$(read_state "$ACTIVE_FILE" "$DEFAULT_THEME")"
  case "$_theme" in
    *-float) return 0 ;;
    *) return 1 ;;
  esac
}

read_state() {
  _file="$1"
  _fallback="$2"
  if [ -s "$_file" ]; then
    sed -n '1p' "$_file"
  else
    printf '%s\n' "$_fallback"
  fi
}

read_state_values() {
  GAPS_IN=""
  GAPS_OUT=""
  WAYBAR=""
  WAYBAR_POS=""
  [ -f "$SPACES_FILE" ] || return 0
  while IFS= read -r _rs_line; do
    [ -n "$_rs_line" ] || continue
    _rs_key="${_rs_line%%=*}"
    _rs_val="${_rs_line#*=}"
    case "$_rs_key" in
      gaps_in)    GAPS_IN="$_rs_val" ;;
      gaps_out)   GAPS_OUT="$_rs_val" ;;
      waybar)     WAYBAR="$_rs_val" ;;
      waybar_pos) WAYBAR_POS="$_rs_val" ;;
    esac
  done < "$SPACES_FILE"
}

write_spaces() {
  mkdir -p "$STATE_DIR"
  {
    [ -n "$GAPS_IN" ]    && printf 'gaps_in=%s\n' "$GAPS_IN"
    [ -n "$GAPS_OUT" ]   && printf 'gaps_out=%s\n' "$GAPS_OUT"
    [ -n "$WAYBAR" ]     && printf 'waybar=%s\n' "$WAYBAR"
    [ -n "$WAYBAR_POS" ] && printf 'waybar_pos=%s\n' "$WAYBAR_POS"
  } > "$SPACES_FILE"
}

compute_defaults() {
  _theme="$(read_state "$ACTIVE_FILE" "$DEFAULT_THEME")"
  _theme_file="$THEMES_DIR/$_theme/theme.lua"

  GAPS_IN_DEF="$(sed -n 's/.*[^[:alnum:]_]gaps_in[[:space:]]*=[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$_theme_file" 2>/dev/null | head -n1)"
  GAPS_OUT_DEF="$(sed -n 's/.*[^[:alnum:]_]gaps_out[[:space:]]*=[[:space:]]*\([0-9][0-9]*\).*/\1/p' "$_theme_file" 2>/dev/null | head -n1)"
  [ -n "${GAPS_IN_DEF:-}" ] || GAPS_IN_DEF=3
  [ -n "${GAPS_OUT_DEF:-}" ] || GAPS_OUT_DEF=1

  case "$_theme" in
    *-float) WAYBAR_DEF=20 ;;
    *)       WAYBAR_DEF=0  ;;
  esac
}

effective_values() {
  compute_defaults
  read_state_values
  [ -n "$GAPS_IN" ]    || GAPS_IN="$GAPS_IN_DEF"
  [ -n "$GAPS_OUT" ]   || GAPS_OUT="$GAPS_OUT_DEF"
  [ -n "$WAYBAR" ]     || WAYBAR="$WAYBAR_DEF"
  [ -n "$WAYBAR_POS" ] || WAYBAR_POS="top"
}

# Drop stored values that fall below the current theme defaults.
clamp_to_defaults() {
  compute_defaults
  read_state_values
  [ -n "$GAPS_IN" ]  && [ "$GAPS_IN"  -lt "$GAPS_IN_DEF" ]  && GAPS_IN=""
  [ -n "$GAPS_OUT" ] && [ "$GAPS_OUT" -lt "$GAPS_OUT_DEF" ] && GAPS_OUT=""
  [ -n "$WAYBAR" ]   && [ "$WAYBAR"   -lt "$WAYBAR_DEF" ]   && WAYBAR=""
  write_spaces
}

apply_waybar_margins() {
  [ -n "${WAYBAR:-}" ] || return 0

  # Ensure the position is resolved (top by default).
  if [ -z "${WAYBAR_POS:-}" ]; then
    compute_defaults
    read_state_values
  fi
  [ -n "${WAYBAR_POS:-}" ] || WAYBAR_POS="top"

  # The "float" look (WAYBAR > 0) uses a subtle negative offset on the far
  # edge; "normal" (WAYBAR = 0) is flush with a small gap. The main bar is
  # mirrored when it sits at the bottom so both top and bottom respect
  # the current float/normal mode.
  if [ "$WAYBAR" -gt 0 ]; then _edge_gap="-8"; else _edge_gap="3"; fi

  # Sysinfo (left, vertical) bar margins stay unchanged regardless of the
  # top/bottom choice — only the main status bar moves.
  sed -i \
    -e "s|\"margin-top\": [0-9-]*|\"margin-top\": $WAYBAR|" \
    -e "s|\"margin-left\": [0-9-]*|\"margin-left\": $WAYBAR|" \
    -e "s|\"margin-bottom\": [0-9-]*|\"margin-bottom\": $WAYBAR|" \
    "$WAYBAR_SYSINFO"

  if [ "$WAYBAR_POS" = "bottom" ]; then
    sed -i \
      -e "s|\"position\": \"[a-z]*\"|\"position\": \"bottom\"|" \
      -e "s|\"margin-bottom\": [0-9-]*|\"margin-bottom\": $WAYBAR|" \
      -e "s|\"margin-top\": [0-9-]*|\"margin-top\": $_edge_gap|" \
      -e "s|\"margin-left\": [0-9-]*|\"margin-left\": $WAYBAR|" \
      -e "s|\"margin-right\": [0-9-]*|\"margin-right\": $WAYBAR|" \
      "$WAYBAR_CFG"
  else
    sed -i \
      -e "s|\"position\": \"[a-z]*\"|\"position\": \"top\"|" \
      -e "s|\"margin-top\": [0-9-]*|\"margin-top\": $WAYBAR|" \
      -e "s|\"margin-bottom\": [0-9-]*|\"margin-bottom\": $_edge_gap|" \
      -e "s|\"margin-left\": [0-9-]*|\"margin-left\": $WAYBAR|" \
      -e "s|\"margin-right\": [0-9-]*|\"margin-right\": $WAYBAR|" \
      "$WAYBAR_CFG"
  fi
}

apply_gaps_runtime() {
  [ "${ARGVUS_NO_RUNTIME:-0}" = 1 ] && return 0
  command -v hyprctl >/dev/null 2>&1 || return 0
  [ -n "${GAPS_IN:-}" ] && hyprctl keyword general:gaps_in "$GAPS_IN" >/dev/null 2>&1
  [ -n "${GAPS_OUT:-}" ] && hyprctl keyword general:gaps_out "$GAPS_OUT" >/dev/null 2>&1
}

restart_waybar() {
  [ "${ARGVUS_NO_RUNTIME:-0}" = 1 ] && return 0
  if command -v waybar >/dev/null 2>&1; then
    sh "$(paths_config hypr/scripts/init.sh)" --waybars >/dev/null 2>&1
  fi
}

set_key() {
  _key="$1"
  _value="$2"
  compute_defaults
  read_state_values

  case "$_key" in
    waybar_pos)
      case "$_value" in
        top|bottom)
          WAYBAR_POS="$_value"
          write_spaces
          effective_values
          apply_waybar_margins
          restart_waybar
          return 0
          ;;
        *)
          printf 'Invalid value: %s (use top|bottom)\n' "$_value" >&2
          return 1
          ;;
      esac
      ;;
  esac

  case "$_key" in
    gaps_in)  _default="$GAPS_IN_DEF" ;;
    gaps_out) _default="$GAPS_OUT_DEF" ;;
    waybar)   _default="$WAYBAR_DEF" ;;
    *) printf 'Invalid key: %s\n' "$_key" >&2; return 1 ;;
  esac

  case "$_value" in
    *[!0-9]*|'') printf 'Invalid value: %s\n' "$_value" >&2; return 1 ;;
  esac
  [ "$_value" -ge "$_default" ] || {
    printf 'Value %s is below the theme minimum %s for %s.\n' "$_value" "$_default" "$_key" >&2
    return 1
  }
  [ "$_value" -le "$MAX_VALUE" ] || {
    printf 'Value %s exceeds maximum %s.\n' "$_value" "$MAX_VALUE" >&2
    return 1
  }

  case "$_key" in
    gaps_in)  GAPS_IN="$_value" ;;
    gaps_out) GAPS_OUT="$_value" ;;
    waybar)   WAYBAR="$_value" ;;
  esac
  write_spaces
}

reset_key() {
  _key="${1:-all}"
  read_state_values
  case "$_key" in
    all)      GAPS_IN=""; GAPS_OUT=""; WAYBAR=""; WAYBAR_POS="" ;;
    gaps_in)  GAPS_IN="" ;;
    gaps_out) GAPS_OUT="" ;;
    waybar)   WAYBAR="" ;;
    waybar_pos) WAYBAR_POS="" ;;
    *) printf 'Invalid key: %s\n' "$_key" >&2; return 1 ;;
  esac
  write_spaces
  compute_defaults
  read_state_values
  GAPS_IN="${GAPS_IN:-$GAPS_IN_DEF}"
  GAPS_OUT="${GAPS_OUT:-$GAPS_OUT_DEF}"
  WAYBAR="${WAYBAR:-$WAYBAR_DEF}"
  WAYBAR_POS="${WAYBAR_POS:-top}"
  apply_gaps_runtime
  apply_waybar_margins
  restart_waybar
}

apply_all() {
  compute_defaults
  read_state_values
  [ -n "$GAPS_IN" ]    || GAPS_IN="$GAPS_IN_DEF"
  [ -n "$GAPS_OUT" ]   || GAPS_OUT="$GAPS_OUT_DEF"
  [ -n "$WAYBAR" ]     || WAYBAR="$WAYBAR_DEF"
  [ -n "$WAYBAR_POS" ] || WAYBAR_POS="top"
  apply_gaps_runtime
  apply_waybar_margins
  restart_waybar
}

print_pairs() {
  printf 'waybar=%s\n' "$WAYBAR"
  printf 'waybar_pos=%s\n' "$WAYBAR_POS"
  printf 'gaps_in=%s\n' "$GAPS_IN"
  printf 'gaps_out=%s\n' "$GAPS_OUT"
}

case "${1:-}" in
  --status)
    effective_values
    print_pairs
    ;;
  --defaults)
    compute_defaults
    WAYBAR="$WAYBAR_DEF"
    WAYBAR_POS="top"
    GAPS_IN="$GAPS_IN_DEF"
    GAPS_OUT="$GAPS_OUT_DEF"
    print_pairs
    ;;
  --get)
    [ -n "${2:-}" ] || { printf 'Missing key\n' >&2; exit 1; }
    effective_values
    case "$2" in
      gaps_in)    printf '%s\n' "$GAPS_IN" ;;
      gaps_out)   printf '%s\n' "$GAPS_OUT" ;;
      waybar)     printf '%s\n' "$WAYBAR" ;;
      waybar_pos) printf '%s\n' "$WAYBAR_POS" ;;
      *) printf 'Invalid key: %s\n' "$2" >&2; exit 1 ;;
    esac
    ;;
  --set)
    [ -n "${2:-}" ] && [ -n "${3:-}" ] || { printf 'Missing key/value\n' >&2; exit 1; }
    set_key "$2" "$3"
    ;;
  --reset)
    reset_key "${2:-all}"
    ;;
  --apply-static)
    if is_float_theme; then
      clamp_to_defaults
      effective_values
      apply_waybar_margins
    else
      # Non-float theme: apply theme defaults without mutating user state.
      compute_defaults
      WAYBAR="$WAYBAR_DEF"
      GAPS_IN="$GAPS_IN_DEF"
      GAPS_OUT="$GAPS_OUT_DEF"
      apply_waybar_margins
    fi
    ;;
  --apply)
    apply_all
    ;;
  *)
    effective_values
    printf 'Usage: spaces-switch.sh [--status|--defaults|--get <key>|--set <key> <value>|--reset [key]|--apply-static|--apply]\n' >&2
    printf 'Current: gaps_in=%s gaps_out=%s waybar=%s waybar_pos=%s\n' \
      "$GAPS_IN" "$GAPS_OUT" "$WAYBAR" "$WAYBAR_POS"
    ;;
esac
