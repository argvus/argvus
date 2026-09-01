#!/usr/bin/env sh
# Configure the inactivity lock timeout used by hypridle.
# Usage: idle-timeout.sh [status|60|300|600|900|1800]
# shellcheck disable=SC1091

set -u

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"
ARGVUS_MUTABLE_CONFIG=1

STATE_DIR="${ARGVUS_CONFIG_HOME}/argvus"
TIMEOUT_FILE="${STATE_DIR}/.idle-timeout"
HYPRIDLE_FILE="$(paths_config hypr/hypridle.conf)"
DEFAULT_TIMEOUT=300
RUNTIME=1

read_timeout() {
  if [ -s "$TIMEOUT_FILE" ]; then
    sed -n '1p' "$TIMEOUT_FILE"
    return 0
  fi

  if [ -f "$HYPRIDLE_FILE" ]; then
    sed -n 's/^[[:space:]]*timeout[[:space:]]*=[[:space:]]*\\([0-9][0-9]*\\).*/\\1/p' "$HYPRIDLE_FILE" | head -n1
    return 0
  fi

  printf '%s\n' "$DEFAULT_TIMEOUT"
}

select_timeout() {
  rofi -config "$(paths_config rofi/config.rasi)" -dmenu -p "Lock timeout" -i -theme-str 'listview {lines: 5;}' <<'EOF'
01 - 1 minute
02 - 5 minutes
03 - 10 minutes
04 - 15 minutes
05 - 30 minutes
EOF
}

normalize_timeout() {
  case "$1" in
    60|1m|1min|*"1 minute"|*"1 minuto") TIMEOUT=60; LABEL="1 min" ;;
    300|5m|5min|*" 5 minutes"|*" 5 minutos") TIMEOUT=300; LABEL="5 min" ;;
    600|10m|10min|*"10 minutes"|*"10 minutos") TIMEOUT=600; LABEL="10 min" ;;
    900|15m|15min|*"15 minutes"|*"15 minutos") TIMEOUT=900; LABEL="15 min" ;;
    1800|30m|30min|*"30 minutes"|*"30 minutos") TIMEOUT=1800; LABEL="30 min" ;;
    *) return 1 ;;
  esac
}

apply_timeout() {
  [ -f "$HYPRIDLE_FILE" ] || {
    printf 'hypridle config not found: %s\n' "$HYPRIDLE_FILE" >&2
    exit 1
  }

  sed -i "0,/^[[:space:]]*timeout[[:space:]]*=.*/s//  timeout = ${TIMEOUT}/" "$HYPRIDLE_FILE"
  mkdir -p "$STATE_DIR"
  printf '%s\n' "$TIMEOUT" > "$TIMEOUT_FILE"
}

refresh_runtime() {
  command -v argvus-sessionctl >/dev/null 2>&1 || return 0
  argvus-sessionctl restart hypridle >/dev/null 2>&1 || true
}

case "${1:-}" in
  status)
    read_timeout
    exit 0
    ;;
  --apply-static)
    RUNTIME=0
    REQUESTED="$(read_timeout)"
    ;;
  '')
    REQUESTED="$(select_timeout)"
    [ -n "$REQUESTED" ] || exit 0
    ;;
  *)
    REQUESTED="$1"
    ;;
esac

if [ "${ARGVUS_NO_RUNTIME:-0}" = 1 ]; then
  RUNTIME=0
fi

if ! normalize_timeout "$REQUESTED"; then
  printf 'Invalid lock timeout: %s\n' "$REQUESTED" >&2
  exit 1
fi

apply_timeout
[ "$RUNTIME" -eq 1 ] && refresh_runtime
notify-send "Lock timeout" "Inactivity lock: ${LABEL}" 2>/dev/null || true
printf '%s\n' "$TIMEOUT"
