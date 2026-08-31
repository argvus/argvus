#!/usr/bin/env sh
# Toggle whether the monitor turns off (dpms) right after the screen is locked.
# Usage: lock-dpms-toggle.sh [status|on|off|toggle]
# shellcheck disable=SC1091

set -u

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

STATE_FILE="$(paths_cache argvus)/lock-dpms"

cmd_status() {
  if [ -f "$STATE_FILE" ]; then
    cat "$STATE_FILE"
  else
    printf 'enabled\n'
  fi
}

cmd_on() {
  mkdir -p "$(dirname "$STATE_FILE")"
  printf 'enabled\n' > "$STATE_FILE"
  printf 'enabled\n'
}

cmd_off() {
  mkdir -p "$(dirname "$STATE_FILE")"
  printf 'disabled\n' > "$STATE_FILE"
  printf 'disabled\n'
}

cmd_toggle() {
  if [ "$(cmd_status)" = "enabled" ]; then
    cmd_off
  else
    cmd_on
  fi
}

case "${1:-}" in
  status) cmd_status ;;
  on)     cmd_on ;;
  off)    cmd_off ;;
  toggle) cmd_toggle ;;
  *)
    echo "Usage: $0 {status|on|off|toggle}" >&2
    exit 1
    ;;
esac
