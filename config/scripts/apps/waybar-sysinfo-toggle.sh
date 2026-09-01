#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

STATE_FILE="$(paths_cache waybar/sysinfo-state)"

ensure_state_dir() {
    mkdir -p "$(dirname "$STATE_FILE")"
}

cmd_on() {
    ensure_state_dir
    echo enabled > "$STATE_FILE"
    argvus-sessionctl restart waybar
    printf "enabled\n"
}

cmd_off() {
    ensure_state_dir
    echo disabled > "$STATE_FILE"
    if command -v systemctl >/dev/null 2>&1; then
        systemctl --user stop argvus-waybar-sysinfo.service >/dev/null 2>&1 || true
    fi
    printf "disabled\n"
}

cmd_status() {
    if [ -f "$STATE_FILE" ]; then
        cat "$STATE_FILE"
    elif command -v systemctl >/dev/null 2>&1 && systemctl --user is-active --quiet argvus-waybar-sysinfo.service 2>/dev/null; then
        printf "enabled\n"
    else
        printf "disabled\n"
    fi
}

cmd_toggle() {
    if [ -f "$STATE_FILE" ] && grep -qx disabled "$STATE_FILE"; then
        cmd_on
    else
        cmd_off
    fi
}

case "$1" in
  on) cmd_on ;;
  off) cmd_off ;;
  status) cmd_status ;;
  toggle) cmd_toggle ;;
  *)
    echo "Usage: $0 {on|off|toggle|status}"
    exit 1
    ;;
esac
