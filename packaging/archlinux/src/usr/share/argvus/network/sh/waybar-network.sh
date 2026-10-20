#!/usr/bin/env sh

set -eu

# shellcheck disable=SC1091
. /usr/share/argvus/lib/i18n.sh

status_output="$(argvus networkctl status)"
value() {
  printf '%s\n' "$status_output" | sed -n "s/^$1=//p" | sed -n '1p'
}

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g; s/[[:cntrl:]]/ /g'
}

status="$(value status)"
iface="$(value iface)"
ssid="$(value ssid)"

case "$status" in
  connected)
    icon='󰱓 '
    title="$(argvus_tr network waybar.connected)"
    class=connected
    ;;
  limited)
    icon='󰍸 '
    title="$(argvus_tr network waybar.limited)"
    class=limited
    ;;
  *)
    icon='󰅛 '
    title="$(argvus_tr network waybar.disconnected)"
    class=disconnected
    ;;
esac

details="$title"
[ -n "$ssid" ] && details="$details — $ssid"
[ -z "$ssid" ] && [ -n "$iface" ] && details="$details — $iface"
[ "$status" = limited ] && details="$details — $(argvus_tr network waybar.no_internet)"

printf '{"text":"%s","class":"%s","tooltip":"%s"}\n' \
  "$(json_escape "$icon")" "$(json_escape "$class")" "$(json_escape "$details")"
