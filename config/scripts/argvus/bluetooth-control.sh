#!/usr/bin/env sh

set -eu

notify() {
  command -v notify-send >/dev/null 2>&1 || return 0
  notify-send "[argvus]:bluetooth" "$1" >/dev/null 2>&1 || true
}

print_manager_status() {
  if command -v blueman-manager >/dev/null 2>&1; then
    printf 'manager=yes\n'
  else
    printf 'manager=no\n'
  fi
}

bluetoothctl_available() {
  command -v bluetoothctl >/dev/null 2>&1
}

btctl() {
  if command -v timeout >/dev/null 2>&1; then
    timeout 3s bluetoothctl "$@"
  else
    bluetoothctl "$@"
  fi
}

bluetooth_show() {
  btctl show 2>/dev/null || true
}

adapter_available() {
  _show="$1"
  [ -n "$_show" ] || return 1
  printf '%s\n' "$_show" | grep -qi "No default controller" && return 1
  printf '%s\n' "$_show" | grep -q "Controller "
}

adapter_value() {
  _show="$1"
  _key="$2"
  printf '%s\n' "$_show" |
    awk -F': ' -v key="$_key" '$1 ~ key { print $2; exit }' |
    sed 's/^[[:space:]]*//'
}

connected_devices() {
  btctl devices Connected 2>/dev/null |
    awk '
      NF {
        $1 = ""
        $2 = ""
        sub(/^[[:space:]]+/, "")
        if ($0 != "") {
          printf "%s%s", sep, $0
          sep = ", "
          count++
        }
      }
      END {
        printf "\n"
      }
    '
}

connected_count() {
  btctl devices Connected 2>/dev/null |
    awk 'NF { count++ } END { print count + 0 }'
}

print_status() {
  if ! bluetoothctl_available; then
    printf 'available=no\n'
    printf 'powered=no\n'
    printf 'connected=0\n'
    printf 'adapter=\n'
    printf 'devices=\n'
    printf 'status=unavailable\n'
    print_manager_status
    return 0
  fi

  _show="$(bluetooth_show)"
  if ! adapter_available "$_show"; then
    printf 'available=no\n'
    printf 'powered=no\n'
    printf 'connected=0\n'
    printf 'adapter=\n'
    printf 'devices=\n'
    printf 'status=no-controller\n'
    print_manager_status
    return 0
  fi

  _powered="$(adapter_value "$_show" "Powered")"
  _alias="$(adapter_value "$_show" "Alias")"
  _devices="$(connected_devices)"
  _connected="$(connected_count)"

  case "$_powered" in
    yes|Yes|true|True)
      _powered="yes"
      if [ "$_connected" -gt 0 ]; then
        _status="connected"
      else
        _status="on"
      fi
    ;;
    *)
      _powered="no"
      _status="off"
    ;;
  esac

  printf 'available=yes\n'
  printf 'powered=%s\n' "$_powered"
  printf 'connected=%s\n' "$_connected"
  printf 'adapter=%s\n' "$_alias"
  printf 'devices=%s\n' "$_devices"
  printf 'status=%s\n' "$_status"
  print_manager_status
}

power_state() {
  _show="$(bluetooth_show)"
  adapter_available "$_show" || {
    printf 'unknown\n'
    return 0
  }
  adapter_value "$_show" "Powered" | tr '[:upper:]' '[:lower:]'
}

try_start_bluetooth_service() {
  command -v systemctl >/dev/null 2>&1 || return 0
  systemctl is-active --quiet bluetooth.service 2>/dev/null && return 0
  if command -v timeout >/dev/null 2>&1; then
    timeout 3s systemctl start bluetooth.service >/dev/null 2>&1 || true
  else
    systemctl start bluetooth.service >/dev/null 2>&1 || true
  fi
}

stop_optional_blueman() {
  command -v systemctl >/dev/null 2>&1 || return 0
  systemctl --user stop argvus-blueman-applet.service >/dev/null 2>&1 || true
}

set_power() {
  _target="$1"

  if ! bluetoothctl_available; then
    notify "bluetoothctl not found"
    print_status
    return 0
  fi

  [ "$_target" = "on" ] && try_start_bluetooth_service

  if btctl power "$_target" >/dev/null 2>&1; then
    [ "$_target" = "off" ] && stop_optional_blueman
  else
    notify "Could not turn Bluetooth $_target"
  fi

  print_status
}

open_manager() {
  if command -v blueman-manager >/dev/null 2>&1; then
    exec blueman-manager
  fi
  notify "Blueman manager is optional and is not installed"
}

case "${1:-status}" in
  status)
    print_status
  ;;
  enable|on)
    set_power on
  ;;
  disable|off)
    set_power off
  ;;
  toggle)
    case "$(power_state)" in
      yes|true) set_power off ;;
      no|false) set_power on ;;
      *) print_status ;;
    esac
  ;;
  manager)
    open_manager
  ;;
  *)
    printf 'usage: %s [status|enable|disable|toggle|manager]\n' "${0##*/}" >&2
    exit 64
  ;;
esac
