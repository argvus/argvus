#!/usr/bin/env sh
# Control display brightness via brightnessctl (laptop backlight) or
# ddcutil (desktop monitors with DDC/CI), with a rofi menu.
# Usage: brightness-switch.sh [--status|--get|--up|--down|--set <percent>|--menu|PERCENT]
# shellcheck disable=SC1091

set -u

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

BACKLIGHT_DEV="${ARGVUS_BACKLIGHT_DEV:-}"
BRIGHTNESS_DEV=""
BACKEND="none"

detect_backend() {
  BACKEND="none"

  if [ -n "$BACKLIGHT_DEV" ]; then
    BRIGHTNESS_DEV="$BACKLIGHT_DEV"
    BACKEND="brightnessctl"
    return 0
  fi

  if command -v brightnessctl >/dev/null 2>&1; then
    _class="$(brightnessctl -l 2>/dev/null | grep "class 'backlight'" | head -1 | sed -E "s/^Device '([^']+)'.*/\1/")"
    if [ -n "$_class" ]; then
      BRIGHTNESS_DEV="$_class"
      BACKEND="brightnessctl"
      return 0
    fi
  fi

  if command -v ddcutil >/dev/null 2>&1 && ddcutil getvcp 10 >/dev/null 2>&1; then
    BACKEND="ddcutil"
    return 0
  fi
}

read_brightnessctl() {
  brightnessctl -m --device "$BRIGHTNESS_DEV" 2>/dev/null \
    | cut -d, -f4 | tr -d '%' | head -1
}

read_ddcutil() {
  _vcp="$(ddcutil getvcp 10 2>/dev/null | grep -i "current value" | head -1)"
  [ -n "$_vcp" ] || { printf '0\n'; return 0; }
  _current="$(printf '%s\n' "$_vcp" | sed -E 's/.*current value[[:space:]]*=[[:space:]]*([0-9]+).*/\1/')"
  _max="$(printf '%s\n' "$_vcp" | sed -E 's/.*max value[[:space:]]*=[[:space:]]*([0-9]+).*/\1/')"
  [ "${_max:-0}" -gt 0 ] || { printf '0\n'; return 0; }
  printf '%s\n' "$((_current * 100 / _max))"
}

read_brightness() {
  case "$BACKEND" in
    brightnessctl) read_brightnessctl ;;
    ddcutil) read_ddcutil ;;
    *) printf '0\n' ;;
  esac
}

apply_brightnessctl() {
  brightnessctl set --device "$BRIGHTNESS_DEV" "$1"
}

apply_ddcutil() {
  _value="$1"
  _base="$(read_ddcutil)"
  case "$_value" in
    +*%) _adj="${_value%\%}"; _target=$((_base + ${_adj#+})) ;;
    *%-) _adj="${_value%-}"; _adj="${_adj%\%}"; _target=$((_base - _adj)) ;;
    *)   _target="${_value%\%}" ;;
  esac
  [ "$_target" -lt 0 ] && _target=0
  [ "$_target" -gt 100 ] && _target=100
  _vcp="$(ddcutil getvcp 10 2>/dev/null | grep -i "current value" | head -1)"
  _max="$(printf '%s\n' "$_vcp" | sed -E 's/.*max value[[:space:]]*=[[:space:]]*([0-9]+).*/\1/')"
  _max="${_max:-100}"
  _abs=$((_max * _target / 100))
  ddcutil setvcp 10 "$_abs"
}

apply() {
  _value="$1"
  case "$BACKEND" in
    none)
      notify_error "Brightness" "no controllable backlight found"
      return 1
      ;;
    brightnessctl) apply_brightnessctl "$_value" ;;
    ddcutil) apply_ddcutil "$_value" ;;
  esac
  _current="$(read_brightness)"
  notify_send "Brightness" "${_current}%"
}

select_menu() {
  _current="$(read_brightness)"
  if locale_is_pt; then
    _prompt="Brilho (atual: ${_current}%)"
    _inc="Aumentar"
    _dec="Diminuir"
  else
    _prompt="Brightness (current: ${_current}%)"
    _inc="Increase"
    _dec="Decrease"
  fi

  _selection=$(
    rofi -config "$(paths_config rofi/config.rasi)" -dmenu -p "$_prompt" -i -theme-str 'listview {lines: 10;}' <<EOF
01 - $_inc +10%
02 - $_inc +5%
03 - $_dec -5%
04 - $_dec -10%
05 - 10%
06 - 25%
07 - 50%
08 - 75%
09 - 90%
10 - 100%
EOF
  )
  [ -z "$_selection" ] && return 1

  case "$_selection" in
    "01 - $_inc +10%") apply "+10%" ;;
    "02 - $_inc +5%")  apply "+5%" ;;
    "03 - $_dec -5%")  apply "5%-" ;;
    "04 - $_dec -10%") apply "10%-" ;;
    *" 10%")  apply "10%" ;;
    *" 25%")  apply "25%" ;;
    *" 50%")  apply "50%" ;;
    *" 75%")  apply "75%" ;;
    *" 90%")  apply "90%" ;;
    *" 100%") apply "100%" ;;
    *) printf 'Invalid brightness selection\n' >&2; return 1 ;;
  esac
}

detect_backend

case "${1:-}" in
  --status) printf '%s\n' "$BACKEND" ;;
  --get)    read_brightness ;;
  --up)     apply "+5%" ;;
  --down)   apply "5%-" ;;
  --set)    [ -n "${2:-}" ] || { printf 'Missing percent\n' >&2; exit 1; }
            apply "$2" ;;
  --menu|"")
    [ "$BACKEND" = "none" ] && {
      notify_error "Brightness" "no controllable backlight found"
      exit 1
    }
    select_menu
  ;;
  *)
    case "$1" in
      +*|*%-|*%) apply "$1" ;;
      *) apply "$1%" ;;
    esac
  ;;
esac
