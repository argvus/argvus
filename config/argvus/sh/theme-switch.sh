#!/usr/bin/env sh
# theme-switch - apply a named theme across the whole argvus desktop
# Usage: theme-switch <theme-name>
# shellcheck disable=SC1091

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"
ARGVUS_MUTABLE_CONFIG=1

THEME="${1:-}"
ACTIVE_FILE="${ARGVUS_CONFIG_HOME}/argvus/.active-theme"
RUNTIME=1
mkdir -p "${ACTIVE_FILE%/*}"

if [ "${ARGVUS_NO_RUNTIME:-0}" = 1 ]; then
  RUNTIME=0
fi

if [ -z "$THEME" ]; then
  THEME=$(
    rofi -config "$(paths_config rofi/config.rasi)" -dmenu -p "   Select Theme" -i -theme-str 'listview {lines: 10;}' <<'EOF'
01 - Argvus Dark
02 - Argvus Dark Float
03 - Argvus Dark Silver
04 - Argvus Dark Silver Float
05 - Argvus Light
06 - Argvus Light Float
07 - Argvus Slate
08 - Argvus Slate Float
09 - Argvus Universe
10 - Argvus Universe Float
EOF
  )

  [ -z "$THEME" ] && exit 0

  case "$THEME" in
    "01 - Argvus Dark")       THEME="argvus-dark" ;;
    "02 - Argvus Dark Float") THEME="argvus-dark-float" ;;
    "03 - Argvus Dark Silver")       THEME="argvus-dark-silver" ;;
    "04 - Argvus Dark Silver Float") THEME="argvus-dark-silver-float" ;;
    "05 - Argvus Light")       THEME="argvus-light" ;;
    "06 - Argvus Light Float") THEME="argvus-light-float" ;;
    "07 - Argvus Slate")       THEME="argvus-slate" ;;
    "08 - Argvus Slate Float") THEME="argvus-slate-float" ;;
    "09 - Argvus Universe")       THEME="argvus-universe" ;;
    "10 - Argvus Universe Float") THEME="argvus-universe-float" ;;
    *) printf 'Invalid theme selection\n' >&2; exit 1 ;;
  esac
fi

HYPR_THEMES="$(paths_config hypr/themes)"
WAYBAR_THEMES="$(paths_config waybar/themes)"
QS_THEMES="$(paths_config quickshell/sidebar-right/themes)"
ROFI_THEMES="$(paths_config rofi/themes)"
ROFI_CONFIG="$(paths_config rofi/config.rasi)"
ROFI_THEME="$(paths_config rofi/theme.rasi)"
ROFI_MODE="$(paths_config rofi/mode.rasi)"
DUNST_THEMES="$(paths_config dunst/themes)"
KITTY_THEMES="$(paths_config kitty/themes)"
BTOP_THEMES="$(paths_config btop/themes)"
BOTTOM_THEMES="$(paths_config bottom/themes)"
YAZI_THEMES="$(paths_config yazi/themes)"
SNAPPY_THEMES="$(paths_config snappy-switcher/themes)"
SUPERFILE_THEMES="$(paths_config superfile/theme)"
QT6CT_COLORS="$(paths_config qt6ct/colors)"
HYPRPAPER_FILE="$(paths_config hypr/hyprpaper.conf)"
HYPRPAPER_DIR="$(paths_backgrounds argvus)"

apply_wallpaper_runtime() {
  _wall="$1"
  [ "$RUNTIME" -eq 1 ] || return 0
  hypr_apply_wallpaper "$_wall"
}

get_active_monitor() {
  if command -v hyprctl >/dev/null 2>&1; then
    hyprctl monitors 2>/dev/null |
      sed -n 's/^Monitor \([^ ]*\).*/\1/p' |
      head -n1
  fi
}

find_theme_wallpaper() {
  _theme="$1"

  case "$_theme" in
    argvus-dark|argvus-dark-float) _wall_name="default.png" ;;
    argvus-dark-silver|argvus-dark-silver-float) _wall_name="argvus-dark-silver.png" ;;
    argvus-light|argvus-light-float) _wall_name="argvus-light.png" ;;
    argvus-slate|argvus-slate-float) _wall_name="argvus-slate.png" ;;
    argvus-universe|argvus-universe-float) _wall_name="argvus-universe.png" ;;
    *) _wall_name="" ;;
  esac

  if [ -n "$_wall_name" ]; then
    _wall="${HYPRPAPER_DIR}/${_wall_name}"
    if [ ! -f "$_wall" ]; then
      printf 'Error: wallpaper not found: %s\n' "$_wall" >&2
      return 1
    fi
    printf '%s\n' "$_wall"
    return 0
  fi

  for _ext in jpeg jpg png webp; do
    _wall="${HYPR_THEMES}/${_theme}/wallpaper.${_ext}"
    [ -f "$_wall" ] && { printf '%s\n' "$_wall"; return 0; }

    _wall="${HYPRPAPER_DIR}/${_theme}.${_ext}"
    [ -f "$_wall" ] && { printf '%s\n' "$_wall"; return 0; }
  done

  # Backward compatibility for older assets with display-case names.
  find "$HYPRPAPER_DIR" -maxdepth 1 -type f -iname "${_theme}.*" | head -n1
}

theme_value() {
  _file="$1"
  _name="$2"
  _fallback="$3"

  if [ -f "$_file" ]; then
    _value=$(sed -n "s|^${_name}[[:space:]]*=[[:space:]]*\"\\{0,1\\}\\([^\" ]*\\)\"\\{0,1\\}.*|\\1|p" "$_file" | head -n1)
    [ -n "$_value" ] && { printf '%s\n' "$_value"; return 0; }
  fi

  printf '%s\n' "$_fallback"
}

set_dunst_section_value() {
  _file="$1"
  _section="$2"
  _key="$3"
  _value="$4"
  _tmp="${_file}.theme.$$"

  awk -v section="[$_section]" -v key="$_key" -v value="    $_key = \"$_value\"" '
    /^\[/ { in_section = ($0 == section) }
    in_section && $0 ~ "^[[:space:]]*" key "[[:space:]]*=" {
      print value
      next
    }
    { print }
  ' "$_file" > "$_tmp" && mv "$_tmp" "$_file"
}

apply_dunst_theme() {
  _theme_file="$DUNST_THEMES/$THEME/dunstrc.theme"
  _dunstrc="$(paths_config dunst/dunstrc)"
  [ -f "$_theme_file" ] && [ -f "$_dunstrc" ] || return 0

  _highlight=$(theme_value "$_theme_file" highlight "#3590bd")
  _frame=$(theme_value "$_theme_file" frame_color "$_highlight")
  _low_bg=$(theme_value "$_theme_file" low_background "#101010")
  _low_fg=$(theme_value "$_theme_file" low_foreground "#aaaaaa")
  _normal_bg=$(theme_value "$_theme_file" normal_background "$_low_bg")
  _normal_fg=$(theme_value "$_theme_file" normal_foreground "$_low_fg")
  _critical_bg=$(theme_value "$_theme_file" critical_background "$_normal_bg")
  _critical_fg=$(theme_value "$_theme_file" critical_foreground "$_normal_fg")
  _app_bg=$(theme_value "$_theme_file" app_background "$_normal_bg")
  _app_fg=$(theme_value "$_theme_file" app_foreground "$_normal_fg")

  set_dunst_section_value "$_dunstrc" global highlight "$_highlight"
  set_dunst_section_value "$_dunstrc" global frame_color "$_frame"

  set_dunst_section_value "$_dunstrc" urgency_low background "$_low_bg"
  set_dunst_section_value "$_dunstrc" urgency_low foreground "$_low_fg"
  set_dunst_section_value "$_dunstrc" urgency_low frame_color "$_frame"

  set_dunst_section_value "$_dunstrc" urgency_normal background "$_normal_bg"
  set_dunst_section_value "$_dunstrc" urgency_normal foreground "$_normal_fg"
  set_dunst_section_value "$_dunstrc" urgency_normal frame_color "$_frame"

  set_dunst_section_value "$_dunstrc" urgency_critical background "$_critical_bg"
  set_dunst_section_value "$_dunstrc" urgency_critical foreground "$_critical_fg"
  set_dunst_section_value "$_dunstrc" urgency_critical frame_color "$_frame"
  set_dunst_section_value "$_dunstrc" urgency_critical highlight "$_highlight"

  for _section in hyprshot volume gpu-screen-recorder network spotify discord; do
    set_dunst_section_value "$_dunstrc" "$_section" background "$_app_bg"
    set_dunst_section_value "$_dunstrc" "$_section" foreground "$_app_fg"
    set_dunst_section_value "$_dunstrc" "$_section" frame_color "$_frame"
    set_dunst_section_value "$_dunstrc" "$_section" highlight "$_highlight"
  done
}

apply_wallpaper() {
  _wall="$1"
  [ -z "$_wall" ] && return 0

  _config_path=$(printf '%s\n' "$_wall" | sed "s|^$HOME|~|")
  _monitor="$(get_active_monitor)"

  if [ -n "$_monitor" ]; then
    sed -i "s|^[[:space:]]*monitor[[:space:]]*=.*$|  monitor = ${_monitor}|" "$HYPRPAPER_FILE"
  fi

  sed -i "s|^[[:space:]]*path[[:space:]]*=.*$|  path =  ${_config_path}|" "$HYPRPAPER_FILE"

  apply_wallpaper_runtime "$_wall"
}

# Sincroniza o tema do argvus-storage com o tema ativo.
# Mapeia: dark -> argvus-dark.css, silver -> argvus-dark-silver.css, slate -> argvus-slate.css, light -> argvus-light.css
apply_argvus_storage_theme() {
  _storage_theme_dir="$(paths_config argvus-storage/themes)"
  _storage_theme_dest="$(paths_config argvus-storage/theme.css)"

  # Tenta encontrar os arquivos de tema em ordem de prioridade:
  # 1. Diretório do usuário (~/.config/argvus-storage/themes)
  # 2. Diretório do sistema (/etc/argvus-storage/themes)
  # 3. Diretório do projeto (para desenvolvimento)
  _theme_src=""
  case "$THEME" in
    argvus-dark|argvus-dark-float)
      _theme_name="argvus-dark.css" ;;
    argvus-dark-silver|argvus-dark-silver-float)
    _theme_name="argvus-dark-silver.css" ;;
    argvus-slate|argvus-slate-float)
      _theme_name="argvus-slate.css" ;;
    argvus-light|argvus-light-float)
      _theme_name="argvus-light.css" ;;
    argvus-universe|argvus-universe-float)
      _theme_name="argvus-universe.css" ;;
    *)
      return 0 ;;
  esac

  # Tenta o diretório do usuário
  if [ -f "${_storage_theme_dir}/${_theme_name}" ]; then
    _theme_src="${_storage_theme_dir}/${_theme_name}"
  # Tenta o sistema
  elif [ -f "/etc/argvus-storage/themes/${_theme_name}" ]; then
    _theme_src="/etc/argvus-storage/themes/${_theme_name}"
  # Tenta o diretório do projeto argvus-storage (desenvolvimento)
  elif [ -f "$(dirname "$0")/../../../../argvus-storage/themes/${_theme_name}" ]; then
    _theme_src="$(dirname "$0")/../../../../argvus-storage/themes/${_theme_name}"
  # Tenta o diretório legado, caso exista em uma instalação antiga.
  elif [ -f "$(paths_config argvus-storage/themes/${_theme_name})" ]; then
    _theme_src="$(paths_config argvus-storage/themes/${_theme_name})"
  else
    return 0
  fi

  mkdir -p "$(dirname "$_storage_theme_dest")"
  cp "$_theme_src" "$_storage_theme_dest"
}

# Sincroniza o tema do argvus-calendar com o tema ativo.
# O destino fica no cache do usuário para a troca de tema não precisar de sudo.
apply_argvus_calendar_theme() {
  _calendar_theme_cache="${XDG_CACHE_HOME:-$HOME/.cache}/argvus-calendar/theme.css"
  _calendar_theme_name=""

  case "$THEME" in
    argvus-dark|argvus-dark-float|argvus-dark-silver|argvus-dark-silver-float|argvus-slate|argvus-slate-float)
      _calendar_theme_name="${THEME}.css" ;;
    argvus-universe|argvus-universe-float)
      _calendar_theme_name="${THEME}.css" ;;
    *)
      return 0 ;;
  esac

  _calendar_theme_src=""
  if [ -f "$(paths_config argvus-calendar/themes/${_calendar_theme_name})" ]; then
    _calendar_theme_src="$(paths_config argvus-calendar/themes/${_calendar_theme_name})"
  elif [ -f "/etc/argvus-calendar/themes/${_calendar_theme_name}" ]; then
    _calendar_theme_src="/etc/argvus-calendar/themes/${_calendar_theme_name}"
  elif [ -f "$(dirname "$0")/../../../../argvus-calendar/resources/themes/${_calendar_theme_name}" ]; then
    _calendar_theme_src="$(dirname "$0")/../../../../argvus-calendar/resources/themes/${_calendar_theme_name}"
  else
    return 0
  fi

  mkdir -p "$(dirname "$_calendar_theme_cache")"
  cp "$_calendar_theme_src" "$_calendar_theme_cache"
  command -v argvus-calendar >/dev/null 2>&1 && argvus-calendar reload >/dev/null 2>&1 || true
}

if [ -z "$THEME" ]; then
  printf 'Usage: theme-switch <theme-name>\n' >&2
  exit 1
fi

for _dir in \
  "$HYPR_THEMES/$THEME" \
  "$WAYBAR_THEMES/$THEME" \
  "$QS_THEMES/$THEME" \
  "$ROFI_THEMES/$THEME" \
  "$DUNST_THEMES/$THEME" \
  "$KITTY_THEMES/$THEME" \
  "$BTOP_THEMES/$THEME" \
  "$SNAPPY_THEMES/$THEME"; do
  if [ ! -d "$_dir" ]; then
    printf 'Error: theme directory not found: %s\n' "$_dir" >&2
    exit 1
  fi
 done

if [ ! -f "$SUPERFILE_THEMES/$THEME.toml" ]; then
  printf 'Warning: superfile theme not found: %s\n' "$SUPERFILE_THEMES/$THEME.toml" >&2
fi

if [ ! -f "$QT6CT_COLORS/$THEME.conf" ]; then
  printf 'Warning: qt6ct color scheme not found: %s\n' "$QT6CT_COLORS/$THEME.conf" >&2
fi

if ! _theme_wallpaper="$(find_theme_wallpaper "$THEME")"; then
  if [ "$RUNTIME" -eq 1 ]; then
    exit 1
  fi
  _theme_wallpaper=""
fi

printf '%s' "$THEME" > "$ACTIVE_FILE"

# ----- Per-theme waybar layout -----
_waybar_cfg="$(paths_config waybar/config.jsonc)"
_waybar_cfg_sysinfo="$(paths_config waybar/sysinfo.jsonc)"
_sysinfo_css="$(paths_config waybar/sysinfo.css)"

case "$THEME" in
  argvus-dark | argvus-dark-silver | argvus-light | argvus-slate | argvus-universe)
    sed -i "s|\"margin-top\": [0-9]*|\"margin-top\": 0|" "$_waybar_cfg"
    sed -i "s|\"margin-left\": [0-9]*|\"margin-left\": 0|" "$_waybar_cfg"
    sed -i "s|\"margin-right\": [0-9]*|\"margin-right\": 0|" "$_waybar_cfg"
    sed -i "s|\"margin-bottom\": -\?[0-9]*|\"margin-bottom\": 3|" "$_waybar_cfg"
    sed -i "s|\"margin-top\": -\?[0-9]*|\"margin-top\": 1|" "$_waybar_cfg_sysinfo"
    sed -i "s|\"margin-left\": -\?[0-9]*|\"margin-left\": 1|" "$_waybar_cfg_sysinfo"
    sed -i "s|\"margin-bottom\": -\?[0-9]*|\"margin-bottom\": 1|" "$_waybar_cfg_sysinfo"
    sed -i '/^window#waybar {/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$(paths_config waybar/style.css)"
    sed -i '/^#workspaces button/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$(paths_config waybar/style.css)"
    sed -i '/^#workspaces button\.active,/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$(paths_config waybar/style.css)"
    sed -i '/^tooltip {/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$(paths_config waybar/style.css)"
    sed -i '/#right-0, #right-1, #right-2, #right-search, #mpris {/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$(paths_config waybar/style.css)"
    sed -i '/^window#waybar {/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$_sysinfo_css"
    _rofi_cfg="$(paths_config rofi/theme.rasi)"
    sed -i '/^window {/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$_rofi_cfg"
    sed -i '/^element selected.normal {/,/^}/s/border-radius: [0-9]*px;/border-radius: 0px;/' "$_rofi_cfg"
    ;;
  *)
    sed -i "s|\"margin-top\": [0-9]*|\"margin-top\": 5|" "$_waybar_cfg"
    sed -i "s|\"margin-left\": [0-9]*|\"margin-left\": 20|" "$_waybar_cfg"
    sed -i "s|\"margin-right\": [0-9]*|\"margin-right\": 20|" "$_waybar_cfg"
    sed -i "s|\"margin-bottom\": -\?[0-9]*|\"margin-bottom\": -8|" "$_waybar_cfg"
    sed -i "s|\"margin-top\": -\?[0-9]*|\"margin-top\": 15|" "$_waybar_cfg_sysinfo"
    sed -i "s|\"margin-left\": -\?[0-9]*|\"margin-left\": 20|" "$_waybar_cfg_sysinfo"
    sed -i "s|\"margin-bottom\": -\?[0-9]*|\"margin-bottom\": 15|" "$_waybar_cfg_sysinfo"
    sed -i '/^window#waybar {/,/^}/s/border-radius: [0-9]*px;/border-radius: 4px;/' "$(paths_config waybar/style.css)"
    sed -i '/^#workspaces button/,/^}/s/border-radius: [0-9]*px;/border-radius: 5px;/' "$(paths_config waybar/style.css)"
    sed -i '/^#workspaces button\.active,/,/^}/s/border-radius: [0-9]*px;/border-radius: 4px;/' "$(paths_config waybar/style.css)"
    sed -i '/^tooltip {/,/^}/s/border-radius: [0-9]*px;/border-radius: 8px;/' "$(paths_config waybar/style.css)"
    sed -i '/#right-0, #right-1, #right-2, #right-search, #mpris {/,/^}/s/border-radius: [0-9]*px;/border-radius: 5px;/' "$(paths_config waybar/style.css)"
    sed -i '/^window#waybar {/,/^}/s/border-radius: [0-9]*px;/border-radius: 8px;/' "$_sysinfo_css"
    _rofi_cfg="$(paths_config rofi/theme.rasi)"
    sed -i '/^window {/,/^}/s/border-radius: [0-9]*px;/border-radius: 6px;/' "$_rofi_cfg"
    sed -i '/^element selected.normal {/,/^}/s/border-radius: [0-9]*px;/border-radius: 5px;/' "$_rofi_cfg"
    ;;
esac

case "$THEME" in
  argvus-slate)
    sed -i '/^window#waybar {/,/^}/s/border: .*;/border: none;/' "$(paths_config waybar/style.css)"
    ;;
  *)
    sed -i '/^window#waybar {/,/^}/s/border: .*;/border: 1px solid @th-decorate;/' "$(paths_config waybar/style.css)"
    ;;
esac

sed -i "s|@import url(\"./themes/.*/theme.css\");|@import url(\"./themes/${THEME}/theme.css\");|" \
  "$(paths_config waybar/style.css)"

sed -i "s|@import url(\"./themes/.*/sysinfo-theme.css\");|@import url(\"./themes/${THEME}/sysinfo-theme.css\");|" \
  "$(paths_config waybar/sysinfo.css)"

sed -i "s|rofi -config [^ ]* -show drun|rofi -config ${ROFI_CONFIG} -show drun|" \
  "$_waybar_cfg"

sed -i "s|@theme \".*/rofi/theme.rasi\"|@theme \"${ROFI_THEME}\"|" "$ROFI_CONFIG"

sed -i "s|@import \".*/rofi/themes/.*/theme.rasi\"|@import \"${ROFI_THEMES}/${THEME}/theme.rasi\"|" \
  "$ROFI_THEME"

sed -i "s|@import \".*/rofi/mode.rasi\"|@import \"${ROFI_MODE}\"|" "$ROFI_THEME"

sed -i "s|include .*/kitty/themes/.*/theme.conf|include ${KITTY_THEMES}/${THEME}/theme.conf|" \
  "$(paths_config kitty/kitty.conf)"

apply_dunst_theme

if [ -f "$HYPR_THEMES/$THEME/hyprtoolkit.conf" ]; then
  cp "$HYPR_THEMES/$THEME/hyprtoolkit.conf" "$(paths_config hypr/hyprtoolkit.conf)"
fi

if [ -f "$HYPR_THEMES/$THEME/application-style.conf" ]; then
  cp "$HYPR_THEMES/$THEME/application-style.conf" "$(paths_config hypr/application-style.conf)"
fi

_qt6ct_conf="$(paths_config qt6ct/qt6ct.conf)"
if [ -f "$_qt6ct_conf" ] && [ -f "$QT6CT_COLORS/$THEME.conf" ]; then
  sed -i "s|^color_scheme_path=.*|color_scheme_path=${QT6CT_COLORS}/${THEME}.conf|" "$_qt6ct_conf"
  sed -i "s|^custom_palette=.*|custom_palette=true|" "$_qt6ct_conf"
fi

if [ "$RUNTIME" -eq 1 ] && { [ -f "$HYPR_THEMES/$THEME/hyprtoolkit.conf" ] || [ -f "$HYPR_THEMES/$THEME/application-style.conf" ]; }; then
  systemctl --user set-environment QT_QPA_PLATFORM=wayland QT_QPA_PLATFORMTHEME=qt6ct QT_QUICK_CONTROLS_STYLE=org.hyprland.style
  systemctl --user restart hyprpolkitagent 2>/dev/null || true
fi

if [ -f "$BTOP_THEMES/$THEME/theme.theme" ]; then
  _btop_conf="$(paths_config btop/btop.conf)"
  sed -i "s|color_theme = .*|color_theme = \"${BTOP_THEMES}/${THEME}/theme.theme\"|" "$_btop_conf"
fi

if [ -f "$SNAPPY_THEMES/$THEME/theme.ini" ]; then
  _snappy_conf="$(paths_config snappy-switcher/config.ini)"
  sed -i "s|^name = .*|name = ${THEME}/theme.ini|" "$_snappy_conf"
fi

if [ -f "$BOTTOM_THEMES/$THEME/bottom.toml" ]; then
  cp "$BOTTOM_THEMES/$THEME/bottom.toml" "$(paths_config bottom/bottom.toml)"
fi

if [ -f "$YAZI_THEMES/$THEME/theme.toml" ]; then
  cp "$YAZI_THEMES/$THEME/theme.toml" "$(paths_config yazi/theme.toml)"
fi

_superfile_conf="$(paths_config superfile/config.toml)"
if [ -f "$_superfile_conf" ] && [ -f "$SUPERFILE_THEMES/$THEME.toml" ]; then
  sed -i "s|^theme = .*|theme = \"${THEME}\"|" "$_superfile_conf"
fi

# Reset GTK mode to match the selected theme.
MODE_CSS="$(paths_config waybar/mode.css)"
printf '/* mode.css — reset on theme switch */\n' > "$MODE_CSS"
GTK_MODE_FILE="${ARGVUS_CONFIG_HOME}/argvus/.gtk-mode"
mkdir -p "$(dirname "$GTK_MODE_FILE")"
case "$THEME" in
  argvus-light | argvus-light-float)
    printf 'light\n' > "$GTK_MODE_FILE"
    if command -v gsettings >/dev/null 2>&1; then
      gsettings set org.gnome.desktop.interface color-scheme prefer-light 2>/dev/null || true
      gsettings set org.gnome.desktop.interface gtk-theme Adwaita 2>/dev/null || true
    fi
    ;;
  *)
    printf 'dark\n' > "$GTK_MODE_FILE"
    if command -v gsettings >/dev/null 2>&1; then
      gsettings set org.gnome.desktop.interface color-scheme prefer-dark 2>/dev/null || true
      gsettings set org.gnome.desktop.interface gtk-theme Adwaita-dark 2>/dev/null || true
    fi
    ;;
esac

# Every theme owns its default accent. A manual accent remains active only until
# the user switches themes, including when switching back to the same theme.
if ! sh "$(paths_config argvus/sh/accent-switch.sh)" --theme-default; then
  printf 'Error: could not restore the default accent for %s.\n' "$THEME" >&2
  exit 1
fi

if ! sh "$(paths_config argvus/sh/hyprlock-theme.sh)" --invalidate; then
  printf 'Error: could not apply the Hyprlock theme for %s.\n' "$THEME" >&2
  exit 1
fi

# Re-apply or reset spaces override depending on theme type (float vs non-float).
case "$THEME" in
  *-float)
    # Float themes: re-apply user's spaces override on top of theme defaults.
    if ! sh "$(paths_config argvus/sh/spaces-switch.sh)" --apply-static; then
      printf 'Error: could not re-apply the spaces override for %s.\n' "$THEME" >&2
      exit 1
    fi
    ;;
  *)
    # Non-float themes: reset spaces to theme defaults (clear user overrides).
    if ! sh "$(paths_config argvus/sh/spaces-switch.sh)" --reset all; then
      printf 'Error: could not reset spaces for %s.\n' "$THEME" >&2
      exit 1
    fi
    ;;
esac

if [ "$RUNTIME" -eq 1 ]; then
  # Reload Hyprland config
  hyprctl reload

  # Restart waybar with new theme CSS (mode.css is now clean/dark)
  sh "$(paths_config hypr/scripts/init.sh)" --waybars
fi

# Set wallpaper for the new theme
apply_wallpaper "$_theme_wallpaper"

if [ "$RUNTIME" -eq 1 ]; then
  # Restart dunst with new theme colors
  pkill -x dunst 2>/dev/null || true
  if command -v dunst >/dev/null 2>&1; then
    dunst -config "$(paths_config dunst/dunstrc)" &
  fi

  # Restart snappy-switcher with new theme
  pkill snappy-switcher 2>/dev/null || true
  sleep 0.2
  snappy-switcher --daemon &

  # Signal running kitty instances to reload config (SIGUSR1)
  for _pid in $(pgrep -x kitty 2>/dev/null); do
    kill -USR1 "$_pid" 2>/dev/null || true
  done
fi

# Sidebar NOT restarted — Theme.qml picks up the new theme dynamically
# via FileView watching .active-theme.

apply_argvus_storage_theme
apply_argvus_calendar_theme

[ "$RUNTIME" -eq 1 ] && notify-send "Theme" "Switched to '${THEME}'" 2>/dev/null || true
printf "Theme '%s' applied.\n" "$THEME"
