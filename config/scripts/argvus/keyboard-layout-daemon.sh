#!/usr/bin/env sh
#
# Daemon that shows a notification when the keyboard layout changes (BR/US).
#
# Works on any machine: it listens to Hyprland's "activelayout" IPC events and
# notifies once per change, ignoring virtual/control/mouse devices.
#
# Started automatically by init.sh --started.

# Resolve the Hyprland instance socket regardless of session specifics.
SIG="${HYPRLAND_INSTANCE_SIGNATURE:-}"
if [ -z "$SIG" ]; then
  SIG="$(hyprctl instances -j 2>/dev/null | jq -r '.[0].instance' 2>/dev/null)"
fi
SOCKET="${XDG_RUNTIME_DIR:-/tmp}/hypr/${SIG}/.socket2.sock"

if [ ! -S "$SOCKET" ]; then
  printf 'keyboard-layout-daemon: IPC socket not found (%s)\n' "$SOCKET" >&2
  exit 1
fi

# Map the keymap description to a friendly label + flag.
layout_label() {
  case "$1" in
    *us*|*english*) printf 'English (US)  🇺🇸' ;;
    *pt*|*portuguese*|*brazil*) printf 'Portuguese (BR)  🇧🇷' ;;
    *) printf '%s' "$1" ;;
  esac
}

last=""
# Listen for layout events, coalescing the burst emitted per keyboard.
socat -u "UNIX-CONNECT:${SOCKET}" - 2>/dev/null \
  | grep --line-buffered 'activelayout>>' \
  | while IFS= read -r line; do
      dev="${line#*>>}"; dev="${dev%%,*}"
      layout="${line##*,}"
      # Skip virtual/control/mouse/audio devices so we notify only once.
      case "$dev" in
        *power*|*sleep*|*system-control*|*consumer-control*|*mouse*|*audio-device*) continue ;;
      esac
      if [ "$layout" != "$last" ]; then
        last="$layout"
        notify-send "Keyboard layout" "$(layout_label "$layout")"
        # Coalesce the burst emitted simultaneously by several keyboards.
        sleep 0.2
      fi
    done
