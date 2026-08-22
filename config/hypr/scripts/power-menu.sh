#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

do_lock() {
  sh "$(paths_config argvus/sh/hyprlock-theme.sh)" >/dev/null || return 1
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
  # sudo tee /usr/local/bin/veracrypt-security-exit.sh > /dev/null << 'EOF'
  # /usr/bin/veracrypt --text --dismount
  # EOF
  # sudo chmod +x /usr/local/bin/veracrypt-security-exit.sh

  # 2 - Create permissions script:
  # cat << EOF > /etc/sudoers.d/veracrypt-unmount
  # <USER> ALL=(root) NOPASSWD: /usr/local/bin/veracrypt-security-exit.sh
  # EOF

  if [ -f "/usr/local/bin/veracrypt-security-exit.sh" ]; then
    sudo /usr/local/bin/veracrypt-security-exit.sh && notify-send "VeraCrypt" "Umount all devices" 2>/dev/null || true
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
if [ "$FINDER" = "/usr/bin/rofi" ]; then
  CHOICE=$(printf '%s\n' \
    "$LOCK" \
    "$SUSPEND" \
    "$LOGOUT" \
    "$REBOOT" \
    "$SHUTDOWN" |
    $FINDER -config "$(paths_config rofi/config.rasi)" -dmenu -p ">" \
    -theme-str 'window {width: 220px;} listview {lines: 5;}' -no-custom -i)
elif [ "$FINDER" = "/usr/bin/wofi" ]; then
  CHOICE=$(printf '%s\n' \
    "$LOCK" \
    "$SUSPEND" \
    "$LOGOUT" \
    "$REBOOT" \
    "$SHUTDOWN" |
    $FINDER)
fi

# ── Despatch ------------------------------------------------------------------
case "$CHOICE" in
"$LOCK")     do_lock ;;
"$SUSPEND")  systemctl suspend ;;
"$LOGOUT")   do_logout ;; # hyprctl dispatch exit
"$REBOOT")   hyprshutdown -t 'Reiniciando...' --post-cmd 'reboot' ;; # systemctl reboot
"$SHUTDOWN") hyprshutdown -t 'Desligando...' --post-cmd 'shutdown -P 0' ;; # systemctl poweroff
esac
