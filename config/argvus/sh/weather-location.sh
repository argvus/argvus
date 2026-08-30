#!/usr/bin/env sh
# Configure the location used by the Quickshell weather card.
# Usage: weather-location.sh [status|--auto|LOCATION]
# shellcheck disable=SC1091

set -eu

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

STATE_DIR="${ARGVUS_CONFIG_HOME}/argvus"
LOCATION_FILE="${STATE_DIR}/.weather-location"

read_location() {
  if [ -f "$LOCATION_FILE" ]; then
    sed -n '1p' "$LOCATION_FILE"
  fi
}

select_location() {
  _current="$(read_location)"
  if locale_is_pt; then
    _auto="Automatico (por IP)"
    _prompt="Local do clima"
    _message="Digite uma cidade, por exemplo: Sao Paulo, SP"
  else
    _auto="Automatic (by IP)"
    _prompt="Weather location"
    _message="Type a city, for example: London, UK"
  fi

  if [ -n "$_current" ]; then
    _options=$(printf '%s\n%s\n' "$_auto" "$_current")
  else
    _options=$(printf '%s\n' "$_auto")
  fi

  if ! _selection=$(printf '%s' "$_options" | rofi -config "$(paths_config rofi/config.rasi)" -dmenu -i -p "$_prompt" -mesg "$_message"); then
    return 1
  fi

  case "$_selection" in
    "$_auto") printf '\n' ;;
    *) printf '%s\n' "$_selection" ;;
  esac
}

case "${1:-}" in
  status)
    read_location
    exit 0
    ;;
  --auto)
    REQUESTED=""
    ;;
  '')
    REQUESTED="$(select_location)" || exit 0
    ;;
  *)
    REQUESTED="$1"
    ;;
esac

LOCATION=$(printf '%s' "$REQUESTED" | tr -d '\r\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')
if [ "${#LOCATION}" -gt 120 ]; then
  printf 'Weather location is too long.\n' >&2
  exit 1
fi

mkdir -p "$STATE_DIR"
printf '%s\n' "$LOCATION" > "$LOCATION_FILE"

if [ -n "$LOCATION" ]; then
  MESSAGE="$LOCATION"
else
  MESSAGE="Automatic (IP)"
fi
if locale_is_pt; then
  SUMMARY="Clima"
  [ -n "$LOCATION" ] || MESSAGE="Automatico (IP)"
else
  SUMMARY="Weather"
fi

notify-send "$SUMMARY" "$MESSAGE" 2>/dev/null || true
printf '%s\n' "$LOCATION"
