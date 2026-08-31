#!/usr/bin/env sh

# shellcheck disable=SC1090,SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
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
  # Kill existing waybar instances synchronously to avoid launching new
  # instances before the old ones exit (which could create duplicate bars).
  pkill -x waybar 2>/dev/null || true
  pkill -x argvus-storage 2>/dev/null || true
  sleep 0.5

  # INFO System (respects sysinfo-state toggle)
  mkdir -p "$(paths_cache waybar)"
  if [ "$(cat "$(paths_cache waybar/sysinfo-state)" 2>/dev/null)" != "disabled" ]; then
    waybar -c "$(paths_config "waybar/argvus-sysinfo.jsonc")" -s "$(paths_config "waybar/argvus-sysinfo.css")" &
    echo enabled > "$(paths_cache waybar/sysinfo-state)" 2>/dev/null
  fi
  sleep 1

  # Status Bar Top
  waybar -c "$(paths_config "waybar/argvus-taskbar.jsonc")" -s "$(paths_config "waybar/argvus-taskbar.css")" &
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

should_manage_btop_config() {
  _conf="$1"
  [ -f "$_conf" ] || return 0
  _theme="$(sed -n 's/^[[:space:]]*color_theme[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$_conf" | head -n1)"
  case "$_theme" in
    ''|Default|*argvus*) return 0 ;;
    *) return 1 ;;
  esac
}

should_manage_foot_config() {
  _conf="$1"
  [ -f "$_conf" ] || return 0
  grep -q 'argvus.*/foot/themes' "$_conf"
}

sync_foot_config() {
  _theme="$ACTIVE_THEME"
  if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ]; then
    _theme="$(sed -n '1p' "$ARGVUS_CONFIG_HOME/argvus/.active-theme")"
  fi

  _foot_dir="$(paths_user_config foot)"
  _system_foot="$(paths_system_config foot)"

  if [ ! -d "$_foot_dir" ] && [ -d "$_system_foot" ]; then
    mkdir -p "$_foot_dir"
    cp -R "$_system_foot/." "$_foot_dir/"
  elif [ -d "$_system_foot" ]; then
    mkdir -p "$_foot_dir"
    if [ -f "$_system_foot/foot.ini" ] && [ ! -e "$_foot_dir/foot.ini" ]; then
      cp "$_system_foot/foot.ini" "$_foot_dir/foot.ini"
    fi
    if [ -d "$_system_foot/themes/${_theme}" ]; then
      mkdir -p "$_foot_dir/themes/${_theme}"
      cp -R "$_system_foot/themes/${_theme}/." "$_foot_dir/themes/${_theme}/"
    fi
  fi

  if [ -f "$_foot_dir/foot.ini" ] && [ -f "$_foot_dir/themes/${_theme}/theme.ini" ]; then
    sed -i "s|^include = .*/foot/themes/.*/theme.ini|include = ${_foot_dir}/themes/${_theme}/theme.ini|" "$_foot_dir/foot.ini"
  fi

  _native_foot="$ARGVUS_CONFIG_HOME/foot/foot.ini"
  if should_manage_foot_config "$_native_foot" && [ -f "$_foot_dir/themes/${_theme}/theme.ini" ]; then
    mkdir -p "${_native_foot%/*}"
    if [ ! -f "$_native_foot" ] && [ -f "$_foot_dir/foot.ini" ]; then
      cp "$_foot_dir/foot.ini" "$_native_foot"
    fi
    sed -i "s|^include = .*/foot/themes/.*/theme.ini|include = ${_foot_dir}/themes/${_theme}/theme.ini|" "$_native_foot"
  fi

  for _pid in $(pgrep -x foot 2>/dev/null) $(pgrep -x footclient 2>/dev/null); do
    kill -USR1 "$_pid" 2>/dev/null || true
  done
}

sync_btop_config() {
  _theme="$ACTIVE_THEME"
  if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ]; then
    _theme="$(sed -n '1p' "$ARGVUS_CONFIG_HOME/argvus/.active-theme")"
  fi

  _btop_dir="$(paths_user_config btop)"
  _system_btop="$(paths_system_config btop)"

  if [ ! -d "$_btop_dir" ] && [ -d "$_system_btop" ]; then
    mkdir -p "$_btop_dir"
    cp -R "$_system_btop/." "$_btop_dir/"
  elif [ -d "$_system_btop" ]; then
    mkdir -p "$_btop_dir"
    if [ -f "$_system_btop/btop.conf" ] && [ ! -e "$_btop_dir/btop.conf" ]; then
      cp "$_system_btop/btop.conf" "$_btop_dir/btop.conf"
    fi
    if [ -d "$_system_btop/themes/${_theme}" ]; then
      mkdir -p "$_btop_dir/themes/${_theme}"
      cp -R "$_system_btop/themes/${_theme}/." "$_btop_dir/themes/${_theme}/"
    fi
  fi

  if [ -f "$_btop_dir/btop.conf" ] && [ -f "$_btop_dir/themes/${_theme}/theme.theme" ]; then
    sed -i "s|color_theme = .*|color_theme = \"${_btop_dir}/themes/${_theme}/theme.theme\"|" "$_btop_dir/btop.conf"
  fi

  _native_btop="$ARGVUS_CONFIG_HOME/btop/btop.conf"
  if should_manage_btop_config "$_native_btop" && [ -f "$_btop_dir/themes/${_theme}/theme.theme" ]; then
    mkdir -p "${_native_btop%/*}"
    if [ ! -f "$_native_btop" ] && [ -f "$_system_btop/btop.conf" ]; then
      cp "$_system_btop/btop.conf" "$_native_btop"
    fi
    sed -i "s|color_theme = .*|color_theme = \"${_btop_dir}/themes/${_theme}/theme.theme\"|" "$_native_btop"
  fi
}

sync_yazi_config() {
  _theme="$ACTIVE_THEME"
  if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ]; then
    _theme="$(sed -n '1p' "$ARGVUS_CONFIG_HOME/argvus/.active-theme")"
  fi

  _yazi_dir="$(paths_user_config yazi)"
  _system_yazi="$(paths_system_config yazi)"

  if [ ! -d "$_yazi_dir" ] && [ -d "$_system_yazi" ]; then
    mkdir -p "$_yazi_dir"
    cp -R "$_system_yazi/." "$_yazi_dir/"
  elif [ -d "$_system_yazi" ]; then
    mkdir -p "$_yazi_dir"
    for _item in package.toml keymap.toml; do
      if [ -f "$_system_yazi/$_item" ] && [ ! -e "$_yazi_dir/$_item" ]; then
        cp "$_system_yazi/$_item" "$_yazi_dir/$_item"
      fi
    done
    if [ -d "$_system_yazi/flavors" ]; then
      mkdir -p "$_yazi_dir/flavors"
      for _flavor in "$_system_yazi"/flavors/*.yazi; do
        [ -d "$_flavor" ] || continue
        _flavor_dst="$_yazi_dir/flavors/${_flavor##*/}"
        [ -e "$_flavor_dst" ] || cp -R "$_flavor" "$_flavor_dst"
      done
    fi
    if [ -d "$_system_yazi/flavors/${_theme}.yazi" ]; then
      mkdir -p "$_yazi_dir/flavors/${_theme}.yazi"
      cp -R "$_system_yazi/flavors/${_theme}.yazi/." "$_yazi_dir/flavors/${_theme}.yazi/"
    fi
  fi

  if [ -f "$_yazi_dir/flavors/${_theme}.yazi/flavor.toml" ]; then
    printf '[flavor]\ndark = "%s"\n' "$_theme" > "$_yazi_dir/theme.toml"
    export YAZI_CONFIG_HOME="$_yazi_dir"
  elif [ -d "$_system_yazi" ]; then
    export YAZI_CONFIG_HOME="$_system_yazi"
  else
    return 0
  fi

  systemctl --user set-environment YAZI_CONFIG_HOME="$YAZI_CONFIG_HOME" 2>/dev/null || true
  command -v hyprctl >/dev/null 2>&1 && hyprctl setenv "YAZI_CONFIG_HOME,$YAZI_CONFIG_HOME" >/dev/null 2>&1 || true

  _native_yazi="$ARGVUS_CONFIG_HOME/yazi"
  _native_theme="$_native_yazi/theme.toml"
  if [ -f "$_native_theme" ] && grep -q 'argvus-' "$_native_theme" && [ -d "$_yazi_dir/flavors/${_theme}.yazi" ]; then
    mkdir -p "$_native_yazi/flavors/${_theme}.yazi"
    cp -R "$_yazi_dir/flavors/${_theme}.yazi/." "$_native_yazi/flavors/${_theme}.yazi/"
    printf '[flavor]\ndark = "%s"\n' "$_theme" > "$_native_theme"
  fi
}

case "$1" in
  --started)
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ]; then
      _argvus_active_theme="$(sed -n '1p' "$ARGVUS_CONFIG_HOME/argvus/.active-theme")"
      ARGVUS_NO_RUNTIME=1 sh "$(paths_config scripts/argvus/theme-switch.sh)" "$_argvus_active_theme" >/dev/null 2>&1 || true
    else
      # First login for this user: apply the packaged default theme so the
      # mutable per-user configs (qt6ct.conf, waybar, rofi, dunst, ...) are
      # lazily materialized from /usr/share/argvus automatically. No
      # argvus --setup --copy-all needed for the DE to be fully themed.
      ARGVUS_NO_RUNTIME=1 sh "$(paths_config scripts/argvus/theme-switch.sh)" "$ACTIVE_THEME" >/dev/null 2>&1 || true
    fi
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.accent-color" ]; then
      sh "$(paths_config scripts/argvus/accent-switch.sh)" --startup
    fi
    set_gsettings
    sync_foot_config
    sync_btop_config
    sync_yazi_config
    start_wallpaper
    run_hypridle

    # Apply user's spaces override (gaps + waybar margins) before starting bars.
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ] || [ -f "$ARGVUS_CONFIG_HOME/argvus/.spaces" ]; then
      sh "$(paths_config scripts/argvus/spaces-switch.sh)" --apply-static
    fi

    # Apply saved monitor scale settings (monitor-switch) and any layout
    # written by nwg-displays.
    sh "$(paths_config scripts/argvus/monitor-switch.sh)" --apply 2>/dev/null || true

    run_waybars

    # PolicyKit agent — set Qt env for systemd services (hyprpolkitagent)
    # AND export for child processes (quickshell).
    export QT_QPA_PLATFORM=wayland QT_QPA_PLATFORMTHEME=qt6ct QT_QUICK_CONTROLS_STYLE=org.hyprland.style
    systemctl --user set-environment QT_QPA_PLATFORM="$QT_QPA_PLATFORM" QT_QPA_PLATFORMTHEME="$QT_QPA_PLATFORMTHEME" QT_QUICK_CONTROLS_STYLE="$QT_QUICK_CONTROLS_STYLE"
    systemctl --user start hyprpolkitagent

    pkill qs; mkdir -p "$(paths_cache quickshell)"; qs -c argvus-control-panel >> "$(paths_cache quickshell)/argvus-control-panel.log" 2>&1 &
    pkill snappy-switcher; snappy-switcher --daemon &
    wl-paste --type text --watch cliphist store &
    wl-paste --type image --watch cliphist store &
    pkill -f keyboard-layout-daemon.sh 2>/dev/null || true
    mkdir -p "$(paths_cache hypr)"
    sh "$(paths_config scripts/argvus/keyboard-layout-daemon.sh)" </dev/null >>"$(paths_cache hypr)/keyboard-layout-daemon.log" 2>&1 &
    run_dunst

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
    sh "$(paths_config scripts/argvus/hyprlock-theme.sh)" --invalidate >/dev/null 2>&1 || true
    sync_foot_config
    sync_btop_config
    sync_yazi_config
    start_wallpaper

    run_hypridle

    systemctl --user restart xdg-desktop-portal-gtk

    # Apply spaces override (waybar margins to config files) before hyprctl reload.
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ] || [ -f "$ARGVUS_CONFIG_HOME/argvus/.spaces" ]; then
      sh "$(paths_config scripts/argvus/spaces-switch.sh)" --apply-static
    fi

    # Reload Hyprland config (applies gaps from .spaces via hyprland.lua)
    hyprctl reload

    # Apply monitor layout written by nwg-displays (if changed).
    sh "$(paths_config scripts/argvus/monitor-switch.sh)" --apply-nwg 2>/dev/null || true

    # Restart waybar with new margins after hyprctl reload.
    run_waybars

    run_dunst

    # PolicyKit agent — set Qt env for systemd services AND child processes.
    export QT_QPA_PLATFORM=wayland QT_QPA_PLATFORMTHEME=qt6ct QT_QUICK_CONTROLS_STYLE=org.hyprland.style
    systemctl --user set-environment QT_QPA_PLATFORM="$QT_QPA_PLATFORM" QT_QPA_PLATFORMTHEME="$QT_QPA_PLATFORMTHEME" QT_QUICK_CONTROLS_STYLE="$QT_QUICK_CONTROLS_STYLE"
    systemctl --user start hyprpolkitagent

    pkill qs; mkdir -p "$(paths_cache quickshell)"; qs -c argvus-control-panel >> "$(paths_cache quickshell)/argvus-control-panel.log" 2>&1 &

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
