#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

# To use this script, you must create the following file:
#
# sudo cat << EOF > /etc/sudoers.d/network-commands
# $USER ALL=(root) NOPASSWD: /sbin/ip, /bin/ip
# EOF

msg() {
  if locale_is_pt; then
    case "$1" in
      no_iface)      echo "Nenhuma interface encontrada" ;;
      network)       echo "Rede" ;;
      disconnected)  echo "Desconectado" ;;
      reconnecting)  echo "Reconectando" ;;
      state)         echo "Estado" ;;
    esac
  else
    case "$1" in
      no_iface)      echo "No interface found" ;;
      network)       echo "Network" ;;
      disconnected)  echo "Disconnected" ;;
      reconnecting)  echo "Reconnecting" ;;
      state)         echo "State" ;;
    esac
  fi
}

IFACE=$(ip route show default 2>/dev/null | awk 'NR==1{print $5}')

if [ -z "$IFACE" ]; then
  IFACE=$(ip link show 2>/dev/null |
    awk -F': ' '/^[0-9]+: e/{print $2; exit}')
fi

[ -z "$IFACE" ] &&
  notify-send "[waybar]:netctl.sh" "$(msg no_iface)" &&
  exit 1

STATE=$(ip link show "$IFACE" 2>/dev/null | awk 'NR==1{print $9}')

case "$STATE" in
UP)
  if sudo -n ip link set "$IFACE" down; then
    notify-send "$(msg network)" "$(msg disconnected) ($IFACE)"
  else
    notify-send "[waybar]:netctl.sh" "sudo ip link set $IFACE down failed"
    exit 1
  fi
  ;;
DOWN | UNKNOWN | "")
  if sudo -n ip link set "$IFACE" up; then
    notify-send "$(msg network)" "$(msg reconnecting) $IFACE..."
  else
    notify-send "[waybar]:netctl.sh" "sudo ip link set $IFACE up failed"
    exit 1
  fi
  ;;
*)
  notify-send "[waybar]:netctl.sh" "$(msg state): $STATE ($IFACE)"
  ;;
esac
