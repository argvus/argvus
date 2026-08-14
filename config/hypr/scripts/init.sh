#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

# Run xdg-user
xdg-user-dirs-update

start_wallpaper() {
  hypr_apply_wallpaper "$WALLPAPER_PATH"
}

set_gsettings() {
  # GTK Theme
  if command -v gsettings >/dev/null 2>&1; then
    if
      gsettings set org.gnome.desktop.interface icon-theme "$ICON_THEME" &&
      gsettings set org.gnome.desktop.interface gtk-theme "$GTK_THEME" &&
      gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark' &&
      gsettings set org.gnome.desktop.interface cursor-theme "$GTK_CURSOR"
    then
      printf "GTK theme applied."
    else
      printf "Could not apply GTK theme."
    fi
  else
    printf "gsettings not found — GTK theme not changed."
  fi

  # Disabled buttons: minimize,maximize,close.
  if
    gsettings set org.gnome.desktop.wm.preferences button-layout "$BUTTON_LAYOUT"
  then
    printf "Disabled buttons 'minimize,maximize,close' in window"
  fi
}

run_waybars() {
  pkill -x waybar &
  pkill -x argvus-storage 2>/dev/null || true
  sleep 0.5

  # INFO System (respects sysinfo-state toggle)
  mkdir -p "$(paths_cache waybar)"
  if [ "$(cat "$(paths_cache waybar/sysinfo-state)" 2>/dev/null)" != "disabled" ]; then
    waybar -c "$(paths_config "waybar/sysinfo.jsonc")" -s "$(paths_config "waybar/sysinfo.css")" &
    echo enabled > "$(paths_cache waybar/sysinfo-state)" 2>/dev/null
  fi
  sleep 1

  # Status Bar Top
  waybar -c "$(paths_config "waybar/config.jsonc")" -s "$(paths_config "waybar/style.css")" &
}

run_hypridle() {
  pkill -x hypridle 2>/dev/null || true
  command -v hypridle >/dev/null 2>&1 || return 0
  hypridle -c "$(paths_config hypr/hypridle.conf)" &
}

run_dunst() {
  pkill -x dunst 2>/dev/null || true
  command -v dunst >/dev/null 2>&1 || return 0
  dunst -config "$(paths_config dunst/dunstrc)" &
}

case "$1" in
  --started)
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.accent-color" ]; then
      sh "$(paths_config argvus/sh/accent-switch.sh)" --startup
    fi
    set_gsettings
    start_wallpaper
    run_hypridle

    # Apply user's spaces override (gaps + waybar margins) before starting bars.
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ] || [ -f "$ARGVUS_CONFIG_HOME/argvus/.spaces" ]; then
      sh "$(paths_config argvus/sh/spaces-switch.sh)" --apply-static
    fi

    run_waybars
    pkill qs; qs -c sidebar-right &
    pkill snappy-switcher; snappy-switcher --daemon &
    wl-paste --type text --watch cliphist store &
    wl-paste --type image --watch cliphist store &
    run_dunst

    # PolicyKit agent (graphical auth) Hyprland
    systemctl --user set-environment QT_QPA_PLATFORM=wayland QT_QPA_PLATFORMTHEME=qt6ct QT_QUICK_CONTROLS_STYLE=org.hyprland.style
    systemctl --user start hyprpolkitagent

    # Bluetooth
    systemctl enable --now bluetooth >/dev/null 2>&1 &
    blueman-applet &
  ;;
  --waybars)
    run_waybars
  ;;
  --set-wallpaper)
    # set_wallpaper
  ;;
  --reload)
    sh "$(paths_config argvus/sh/hyprlock-theme.sh)" --invalidate >/dev/null 2>&1 || true
    start_wallpaper

    run_hypridle

    systemctl --user restart xdg-desktop-portal-gtk

    # Apply spaces override (waybar margins to config files) before hyprctl reload.
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ] || [ -f "$ARGVUS_CONFIG_HOME/argvus/.spaces" ]; then
      sh "$(paths_config argvus/sh/spaces-switch.sh)" --apply-static
    fi

    # Reload Hyprland config (applies gaps from .spaces via hyprland.lua)
    hyprctl reload

    # Restart waybar with new margins after hyprctl reload.
    run_waybars

    run_dunst

    # PolicyKit agent (graphical auth) Hyprland
    systemctl --user set-environment QT_QPA_PLATFORM=wayland QT_QPA_PLATFORMTHEME=qt6ct QT_QUICK_CONTROLS_STYLE=org.hyprland.style
    systemctl --user start hyprpolkitagent

    pkill snappy-switcher 2>/dev/null || true
    sleep 0.2
    snappy-switcher --daemon &

    # Bluetooth
    #systemctl enable --now bluetooth >/dev/null 2>&1 &
    #blueman-applet &

  ;;
  *)
    notify-send "Error" "[hyprland:scripts:init]: Invalid parameter"
  ;;
esac
