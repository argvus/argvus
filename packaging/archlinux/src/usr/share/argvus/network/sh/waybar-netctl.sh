#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/session/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"
# shellcheck source=/usr/share/argvus/lib/i18n.sh
. /usr/share/argvus/lib/i18n.sh

# To use this script, you must create the following file:
#
# sudo cat << EOF > /etc/sudoers.d/network-commands
# $USER ALL=(root) NOPASSWD: /sbin/ip, /bin/ip
# EOF

msg() {
  argvus_tr network "waybar.$1"
}

IFACE=$(ip route show default 2>/dev/null | awk 'NR==1{print $5}')

if [ -z "$IFACE" ]; then
  IFACE=$(ip link show 2>/dev/null |
    awk -F': ' '/^[0-9]+: e/{print $2; exit}')
fi

[ -z "$IFACE" ] &&
  notify-send "$(msg network)" "$(msg no_iface)" &&
  exit 1

STATE=$(ip link show "$IFACE" 2>/dev/null | awk 'NR==1{print $9}')

case "$STATE" in
UP)
  if sudo -n ip link set "$IFACE" down; then
    notify-send "$(msg network)" "$(msg disconnected) ($IFACE)"
  else
    notify-send "$(msg network)" "$(argvus_tr network waybar.command_failed command="sudo ip link set $IFACE down")"
    exit 1
  fi
  ;;
DOWN | UNKNOWN | "")
  if sudo -n ip link set "$IFACE" up; then
    notify-send "$(msg network)" "$(msg reconnecting) $IFACE..."
  else
    notify-send "$(msg network)" "$(argvus_tr network waybar.command_failed command="sudo ip link set $IFACE up")"
    exit 1
  fi
  ;;
*)
  notify-send "$(msg network)" "$(msg state): $STATE ($IFACE)"
  ;;
esac
