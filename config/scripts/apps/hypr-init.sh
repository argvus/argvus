#!/usr/bin/env sh

# shellcheck disable=SC1090,SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

sessionctl() {
  command -v argvus-sessionctl >/dev/null 2>&1 || return 127
  argvus-sessionctl "$@"
}

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

  if command -v gsettings >/dev/null 2>&1; then
    # Disabled buttons: minimize,maximize,close.
    if
      gsettings set org.gnome.desktop.wm.preferences button-layout "$BUTTON_LAYOUT"
    then
      printf "Disabled buttons 'minimize,maximize,close' in window"
    fi
  fi
}

run_waybars() {
  sessionctl restart waybar
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

prepare_session() {
    command -v xdg-user-dirs-update >/dev/null 2>&1 && xdg-user-dirs-update

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

    # Apply user's spaces override (gaps + waybar margins) before bars start.
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ] || [ -f "$ARGVUS_CONFIG_HOME/argvus/.spaces" ]; then
      sh "$(paths_config scripts/argvus/spaces-switch.sh)" --apply-static
    fi

    # Apply saved monitor scale settings (monitor-switch) and any layout
    # written by nwg-displays.
    sh "$(paths_config scripts/argvus/monitor-switch.sh)" --apply 2>/dev/null || true

    # PolicyKit agent and Quickshell inherit these through systemd user env.
    export QT_QPA_PLATFORM=wayland QT_QPA_PLATFORMTHEME=qt6ct QT_QUICK_CONTROLS_STYLE=org.hyprland.style
    sessionctl import-environment >/dev/null 2>&1 || true
}

reload_config() {
    sh "$(paths_config scripts/argvus/hyprlock-theme.sh)" --invalidate >/dev/null 2>&1 || true
    sync_foot_config
    sync_btop_config
    sync_yazi_config

    # Apply spaces override (waybar margins to config files) before hyprctl reload.
    if [ -f "$ARGVUS_CONFIG_HOME/argvus/.active-theme" ] || [ -f "$ARGVUS_CONFIG_HOME/argvus/.spaces" ]; then
      sh "$(paths_config scripts/argvus/spaces-switch.sh)" --apply-static
    fi

    # Reload Hyprland config (applies gaps from .spaces via hyprland.lua)
    hyprctl reload

    # Apply monitor layout written by nwg-displays (if changed).
    sh "$(paths_config scripts/argvus/monitor-switch.sh)" --apply-nwg 2>/dev/null || true
}

case "${1:-}" in
  --started)
    sessionctl ready
  ;;
  --prepare)
    prepare_session
  ;;
  --waybars)
    run_waybars
  ;;
  --set-wallpaper)
    start_wallpaper
  ;;
  --reload-config)
    reload_config
  ;;
  --reload)
    sessionctl reload
  ;;
  *)
    notify-send "Error" "[hyprland:scripts:init]: Invalid parameter"
  ;;
esac
