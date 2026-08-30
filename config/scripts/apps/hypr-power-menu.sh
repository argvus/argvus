#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

do_lock() {
  sh "$(paths_config scripts/argvus/hyprlock-theme.sh)" >/dev/null || return 1
  HYPRLOCK_PATH="$(
    sed -n \
      -e "s|^[[:space:]]*path[[:space:]]*=[[:space:]]*~|$HOME|p" \
      -e "s|^[[:space:]]*path[[:space:]]*=[[:space:]]*\\(/.*\\)|\\1|p" \
      "$(paths_config hypr/hyprlock.conf)" |
      head -n1
  )"
  [ -n "$WALLPAPER_PATH" ] && [ -f "$WALLPAPER_PATH" ] || return 1
  [ -n "$HYPRLOCK_PATH" ] || return 1
  mkdir -p "${HYPRLOCK_PATH%/*}"
  if [ ! -f "$HYPRLOCK_PATH" ] || [ "$WALLPAPER_PATH" -nt "$HYPRLOCK_PATH" ]; then
    magick "$WALLPAPER_PATH" \
      -blur 0x2 \
      -fill black -colorize 20% \
      "$HYPRLOCK_PATH"
  fi
  exec hyprlock
}

do_logout() {
  # Support to VeraCrypt Umount devices in logout
  #
  # 1 - Create script:
  # sudo tee /usr/bin/veracrypt-security-exit.sh > /dev/null << 'EOF'
  # /usr/bin/veracrypt --text --dismount
  # EOF
  # sudo chmod +x /usr/bin/veracrypt-security-exit.sh

  # 2 - Create permissions script:
  # cat << EOF > /etc/sudoers.d/veracrypt-unmount
  # <USER> ALL=(root) NOPASSWD: /usr/bin/veracrypt-security-exit.sh
  # EOF

  if [ -f "/usr/bin/veracrypt-security-exit.sh" ] && sudo -n /usr/bin/veracrypt-security-exit.sh 2>/dev/null; then
    notify-send "VeraCrypt" "Umount all devices" 2>/dev/null || true
  fi

  hyprshutdown
}


# -- power-menu.sh --lock
# Also used for the keyboard shortcut Mod+Shift+l
# ------------------------------------------------------------------------------
[ "$1" = "--lock" ] && {
  do_lock
  exit $?
}

# -- Translate -----------------------------------------------------------------
if locale_is_pt; then
  LOCK="Bloquear"
  SUSPEND="Suspender"
  LOGOUT="Sair"
  REBOOT="Reiniciar"
  SHUTDOWN="Desligar"
else
  LOCK="Lock"
  SUSPEND="Suspend"
  LOGOUT="Log Out"
  REBOOT="Reboot"
  SHUTDOWN="Shut Down"
fi

# -- Menu -----------------------------------------------------------------
# Ensure we have a menu launcher; prefer rofi, fall back to wofi
if [ -z "$FINDER" ]; then
  if command -v rofi >/dev/null 2>&1; then
    FINDER=$(command -v rofi)
  elif command -v wofi >/dev/null 2>&1; then
    FINDER=$(command -v wofi)
  else
    echo "No menu launcher (rofi/wofi) found." >&2
    exit 1
  fi
fi

# Use basename so different install paths still match
case "$(basename "$FINDER")" in
rofi)
  CHOICE=$(printf '%s\n' \
    "$LOCK" \
    "$SUSPEND" \
    "$LOGOUT" \
    "$REBOOT" \
    "$SHUTDOWN" |
    "$FINDER" -config "$(paths_config rofi/config.rasi)" -dmenu -p ">" \
    -theme-str 'window {width: 220px;} listview {lines: 5;}' -no-custom -i)
  ;;
wofi)
  CHOICE=$(printf '%s\n' \
    "$LOCK" \
    "$SUSPEND" \
    "$LOGOUT" \
    "$REBOOT" \
    "$SHUTDOWN" |
    "$FINDER")
  ;;
*)
  echo "Unsupported menu launcher: $FINDER" >&2
  exit 1
  ;;
esac

# ── Despatch ------------------------------------------------------------------
case "$CHOICE" in
"$LOCK")     do_lock ;;
"$SUSPEND")  systemctl suspend ;;
"$LOGOUT")   do_logout ;; # hyprctl dispatch exit
"$REBOOT")   hyprshutdown -t 'Reiniciando...' --post-cmd 'reboot' ;; # systemctl reboot
"$SHUTDOWN") hyprshutdown -t 'Desligando...' --post-cmd 'shutdown -P 0' ;; # systemctl poweroff
esac
