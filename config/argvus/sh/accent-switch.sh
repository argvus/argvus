#!/usr/bin/env sh
# Apply one highlight color without changing theme backgrounds.
# Usage: accent-switch.sh [COLOR|--apply|--startup|--theme-default]
# shellcheck disable=SC1091

set -u

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"
ARGVUS_MUTABLE_CONFIG=1

STATE_DIR="${ARGVUS_CONFIG_HOME}/argvus"
ACCENT_FILE="${STATE_DIR}/.accent-color"
ACTIVE_FILE="${STATE_DIR}/.active-theme"
DEFAULT_ACCENT="#3590bd"
DEFAULT_THEME="argvus-dark"
RUNTIME=1
NOTIFY=1

read_state() {
  _state_file="$1"
  _fallback="$2"
  if [ -s "$_state_file" ]; then
    sed -n '1p' "$_state_file"
  else
    printf '%s\n' "$_fallback"
  fi
}

theme_default_accent() {
  case "$1" in
    argvus-dark|argvus-dark-float) printf '#3590bd\n' ;;
    argvus-dark-silver|argvus-dark-silver-float) printf '#595959\n' ;;
    argvus-light|argvus-light-float) printf '#181818\n' ;;
    argvus-slate|argvus-slate-float) printf '#7391a5\n' ;;
    *) return 1 ;;
  esac
}

read_accent_state() {
  _state_theme="$(read_state "$ACTIVE_FILE" "$DEFAULT_THEME")"
  _state_default="$(theme_default_accent "$_state_theme" 2>/dev/null || printf '%s\n' "$DEFAULT_ACCENT")"
  read_state "$ACCENT_FILE" "$_state_default"
}

select_accent() {
  rofi -config "$(paths_config rofi/config.rasi)" -dmenu -p "Accent color" -i -theme-str 'listview {lines: 9;}' <<'EOF'
01 - Blue       #3590bd
02 - Slate Blue #7391a5
03 - Brown      #996548
04 - Green      #17d174
05 - Magenta    #cb17d1
06 - Red        #d1174f
07 - Yellow     #d1ce17
08 - Purple     #9617d1
09 - Silver     #595959
EOF
}

normalize_accent() {
  _requested="$(printf '%s' "$1" | tr 'A-F' 'a-f')"
  case "$_requested" in
    *\#3590bd|3590bd) COLOR="#3590bd"; RED=53; GREEN=144; BLUE=189; ACCENT_TEXT="#111316" ;;
    *\#181818|181818) COLOR="#181818"; RED=24; GREEN=24; BLUE=24; ACCENT_TEXT="#f7f7f7" ;;
    *\#7391a5|7391a5) COLOR="#7391a5"; RED=115; GREEN=145; BLUE=165; ACCENT_TEXT="#111316" ;;
    *\#996548|996548) COLOR="#996548"; RED=153; GREEN=101; BLUE=72; ACCENT_TEXT="#f7f7f7" ;;
    *\#17d174|17d174) COLOR="#17d174"; RED=23; GREEN=209; BLUE=116; ACCENT_TEXT="#111316" ;;
    *\#cb17d1|cb17d1) COLOR="#cb17d1"; RED=203; GREEN=23; BLUE=209; ACCENT_TEXT="#f7f7f7" ;;
    *\#d1174f|d1174f) COLOR="#d1174f"; RED=209; GREEN=23; BLUE=79; ACCENT_TEXT="#f7f7f7" ;;
    *\#d1ce17|d1ce17) COLOR="#d1ce17"; RED=209; GREEN=206; BLUE=23; ACCENT_TEXT="#111316" ;;
    *\#9617d1|9617d1) COLOR="#9617d1"; RED=150; GREEN=23; BLUE=209; ACCENT_TEXT="#f7f7f7" ;;
    *\#595959|595959) COLOR="#595959"; RED=89; GREEN=89; BLUE=89; ACCENT_TEXT="#f7f7f7" ;;
    *) return 1 ;;
  esac
  HEX="${COLOR#\#}"
}

replace_setting() {
  _file="$1"
  _name="$2"
  _value="$3"
  [ -f "$_file" ] || return 0
  sed -i "s|^${_name}[[:space:]]*=.*|${_name} = ${_value}|" "$_file"
}

replace_toml_color() {
  _file="$1"
  _name="$2"
  [ -f "$_file" ] || return 0
  sed -i "s|^${_name}[[:space:]]*=.*|${_name} = \"${COLOR}\"|" "$_file"
}

set_dunst_section_value() {
  _file="$1"
  _section="$2"
  _key="$3"
  _value="$4"
  _tmp="${_file}.accent.$$"

  awk -v section="[$_section]" -v key="$_key" -v value="    $_key = \"$_value\"" '
    /^\[/ { in_section = ($0 == section) }
    in_section && $0 ~ "^[[:space:]]*" key "[[:space:]]*=" {
      print value
      next
    }
    { print }
  ' "$_file" > "$_tmp" && mv "$_tmp" "$_file"
}

apply_theme_references() {
  _rofi_config="$(paths_config rofi/config.rasi)"
  _rofi_theme_file="$(paths_config rofi/theme.rasi)"
  _rofi_mode="$(paths_config rofi/mode.rasi)"
  _rofi_theme="$(paths_config "rofi/themes/${THEME}/theme.rasi")"
  _kitty_theme="$(paths_config "kitty/themes/${THEME}/theme.conf")"

  sed -i "s|@import url(\"./themes/.*/theme.css\");|@import url(\"./themes/${THEME}/theme.css\");|" \
    "$(paths_config waybar/style.css)" "$(paths_config wlogout/style.css)" 2>/dev/null || true
  sed -i "s|@import url(\"./themes/.*/sysinfo-theme.css\");|@import url(\"./themes/${THEME}/sysinfo-theme.css\");|" \
    "$(paths_config waybar/sysinfo.css)" 2>/dev/null || true
  sed -i "s|@theme \".*/rofi/theme.rasi\"|@theme \"${_rofi_theme_file}\"|" \
    "$_rofi_config" 2>/dev/null || true
  sed -i "s|@import \".*/rofi/themes/.*/theme.rasi\"|@import \"${_rofi_theme}\"|" \
    "$_rofi_theme_file" 2>/dev/null || true
  sed -i "s|@import \".*/rofi/mode.rasi\"|@import \"${_rofi_mode}\"|" \
    "$_rofi_theme_file" 2>/dev/null || true
  sed -i "s|include .*/kitty/themes/.*/theme.conf|include ${_kitty_theme}|" \
    "$(paths_config kitty/kitty.conf)" 2>/dev/null || true
  replace_setting "$(paths_config snappy-switcher/config.ini)" name "${THEME}/theme.ini"
  replace_setting "$(paths_config superfile/config.toml)" theme "\"${THEME}\""
  replace_setting "$(paths_config qt6ct/qt6ct.conf)" color_scheme_path "$(paths_config "qt6ct/colors/${THEME}.conf")"
  if [ -f "$(paths_config "btop/themes/${THEME}/theme.theme")" ]; then
    replace_setting "$(paths_config btop/btop.conf)" color_theme "\"$(paths_config "btop/themes/${THEME}/theme.theme")\""
  fi
  if [ -f "$(paths_config "yazi/themes/${THEME}/theme.toml")" ]; then
    cp "$(paths_config "yazi/themes/${THEME}/theme.toml")" "$(paths_config yazi/theme.toml)"
  fi
}

apply_waybar() {
  _theme_css="$(paths_config "waybar/themes/${THEME}/theme.css")"
  _sysinfo_css="$(paths_config "waybar/themes/${THEME}/sysinfo-theme.css")"
  [ -f "$_theme_css" ] && sed -i \
    -e "s|^@define-color th-decorate .*|@define-color th-decorate        ${COLOR};|" \
    -e "s|^@define-color th-decorate-rgba .*|@define-color th-decorate-rgba   rgba(${RED}, ${GREEN}, ${BLUE}, 0.45);|" \
    -e "s|^@define-color th-border-rights .*|@define-color th-border-rights   rgba(${RED}, ${GREEN}, ${BLUE}, 0.25);|" \
    -e "s|^@define-color th-power .*|@define-color th-power           ${COLOR};|" \
    -e "s|^@define-color th-mpris-border .*|@define-color th-mpris-border    rgba(${RED}, ${GREEN}, ${BLUE}, 0.25);|" \
    "$_theme_css"
  [ -f "$_sysinfo_css" ] && sed -i \
    -e "s|^@define-color th-header .*|@define-color th-header        ${COLOR};|" \
    -e "s|^@define-color th-border .*|@define-color th-border        rgba(${RED}, ${GREEN}, ${BLUE}, 0.15);|" \
    -e "s|^@define-color th-border-header .*|@define-color th-border-header rgba(${RED}, ${GREEN}, ${BLUE}, 0.20);|" \
    "$_sysinfo_css"
}

apply_rofi() {
  _file="$(paths_config "rofi/themes/${THEME}/theme.rasi")"
  [ -f "$_file" ] || return 0
  sed -i \
    -e "s|^[[:space:]]*th-fg:.*|    th-fg:            rgb(${RED}, ${GREEN}, ${BLUE});|" \
    -e "s|^[[:space:]]*th-row-alt:.*|    th-row-alt:       rgba(${RED}, ${GREEN}, ${BLUE}, 7%);|" \
    -e "s|^[[:space:]]*th-border-color:.*|    th-border-color:  rgba(${RED}, ${GREEN}, ${BLUE}, 0.36);|" \
    "$_file"
}

apply_qt_palette() {
  _file="$(paths_config "qt6ct/colors/${THEME}.conf")"
  [ -f "$_file" ] || return 0
  _tmp="${_file}.accent.$$"
  awk -v accent="#ff${HEX}" '
    /^(active_colors|disabled_colors|inactive_colors)=/ {
      equals = index($0, "=")
      prefix = substr($0, 1, equals)
      count = split(substr($0, equals + 1), colors, ", ")
      colors[3] = accent; colors[8] = accent; colors[13] = accent; colors[16] = accent
      printf "%s", prefix
      for (i = 1; i <= count; i++) printf "%s%s", (i > 1 ? ", " : ""), colors[i]
      printf "\n"
      next
    }
    { print }
  ' "$_file" > "$_tmp" && mv "$_tmp" "$_file"
}

apply_quickshell() {
  _file="$(paths_config "quickshell/sidebar-right/themes/${THEME}/Theme.qml")"
  [ -f "$_file" ] || return 0
  sed -i \
    -e "s|^[[:space:]]*readonly property color accent:.*|    readonly property color accent:          \"${COLOR}\"|" \
    -e "s|^[[:space:]]*readonly property color accentDim:.*|    readonly property color accentDim:       \"#22${HEX}\"|" \
    -e "s|^[[:space:]]*readonly property color accentMid:.*|    readonly property color accentMid:       \"#55${HEX}\"|" \
    -e "s|^[[:space:]]*readonly property color accentFaint:.*|    readonly property color accentFaint:     \"#0f${HEX}\"|" \
    -e "s|^[[:space:]]*readonly property color accentLight:.*|    readonly property color accentLight:     \"${COLOR}\"|" \
    -e "s|^[[:space:]]*readonly property color fgTitle:.*|    readonly property color fgTitle:         \"${COLOR}\"|" \
    -e "s|^[[:space:]]*readonly property color fgOnAccent:.*|    readonly property color fgOnAccent:      \"${ACCENT_TEXT}\"|" \
    -e "s|^[[:space:]]*readonly property color bgActive:.*|    readonly property color bgActive:        \"#22${HEX}\"|" \
    -e "s|^[[:space:]]*readonly property color border:.*|    readonly property color border:          \"#22${HEX}\"|" \
    -e "s|^[[:space:]]*readonly property color borderStrong:.*|    readonly property color borderStrong:    \"#55${HEX}\"|" \
    -e "s|^[[:space:]]*readonly property color borderItem:.*|    readonly property color borderItem:      \"#0f${HEX}\"|" \
    "$_file"
}

apply_application_colors() {
  _hyprlock="$(paths_config hypr/hyprlock.conf)"
  _hyprtoolkit="$(paths_config hypr/hyprtoolkit.conf)"
  _dunst="$(paths_config dunst/dunstrc)"
  _kitty="$(paths_config "kitty/themes/${THEME}/theme.conf")"
  _wlogout="$(paths_config "wlogout/themes/${THEME}/theme.css")"
  _snappy="$(paths_config "snappy-switcher/themes/${THEME}/theme.ini")"
  _bottom="$(paths_config bottom/bottom.toml)"
  _btop="$(paths_config "btop/themes/${THEME}/theme.theme")"
  _superfile="$(paths_config "superfile/theme/${THEME}.toml")"

  [ -f "$_hyprlock" ] && sed -i "s|^[[:space:]]*outer_color = .*|  outer_color = rgb(${HEX})|" "$_hyprlock"
  [ -f "$_hyprtoolkit" ] && sed -i \
    -e "s|^bright_text = .*|bright_text = 0xFF${HEX}|" \
    -e "s|^accent = .*|accent = 0xFF${HEX}|" "$_hyprtoolkit"
  if [ -f "$_dunst" ]; then
    for _section in global urgency_low urgency_normal urgency_critical hyprshot volume gpu-screen-recorder network spotify discord; do
      set_dunst_section_value "$_dunst" "$_section" frame_color "$COLOR"
      set_dunst_section_value "$_dunst" "$_section" highlight "$COLOR"
    done
  fi
  [ -f "$_kitty" ] && sed -i "s|^active_tab_foreground .*|active_tab_foreground   ${COLOR}|" "$_kitty"
  [ -f "$_wlogout" ] && sed -i \
    -e "s|^@define-color th-accent .*|@define-color th-accent       ${COLOR};|" \
    -e "s|^@define-color th-accent-text .*|@define-color th-accent-text  ${ACCENT_TEXT};|" "$_wlogout"
  [ -f "$_snappy" ] && sed -i \
    -e "s|^border_color .*|border_color  = ${COLOR}ff|" \
    -e "s|^badge_bg .*|badge_bg      = ${COLOR}ff|" "$_snappy"

  for _key in border selected_bg cpu_color; do replace_toml_color "$_bottom" "$_key"; done
  replace_toml_color "$_bottom" selected_text
  [ -f "$_bottom" ] && sed -i "s|^selected_text = .*|selected_text = \"${ACCENT_TEXT}\"|" "$_bottom"

  for _key in title hi_fg proc_misc cpu_box mem_box net_box proc_box temp_start; do
    [ -f "$_btop" ] && sed -i "s|^\[${_key}\]=.*|[${_key}]=\"${COLOR}\"|" "$_btop"
  done

  for _key in file_panel_border_active file_panel_top_directory_icon footer_border_active sidebar_title sidebar_border_active modal_border_active modal_confirm_bg help_menu_hotkey correct hint; do
    replace_toml_color "$_superfile" "$_key"
  done
  [ -f "$_superfile" ] && sed -i \
    -e "s|^gradient_color = \[\"#[0-9A-Fa-f]*\",|gradient_color = [\"${COLOR}\",|" \
    -e "s|^modal_confirm_fg = .*|modal_confirm_fg = \"${ACCENT_TEXT}\"|" "$_superfile"
}

refresh_runtime() {
  command -v hyprctl >/dev/null 2>&1 && hyprctl reload >/dev/null 2>&1 || true
  if command -v waybar >/dev/null 2>&1; then
    sh "$(paths_config hypr/scripts/init.sh)" --waybars >/dev/null 2>&1 || true
  fi
  pkill -x dunst 2>/dev/null || true
  if command -v dunst >/dev/null 2>&1; then
    dunst -config "$(paths_config dunst/dunstrc)" >/dev/null 2>&1 &
  fi
  if command -v snappy-switcher >/dev/null 2>&1; then
    pkill -x snappy-switcher 2>/dev/null || true
    snappy-switcher --daemon >/dev/null 2>&1 &
  fi
  if command -v qs >/dev/null 2>&1 && pgrep -x qs >/dev/null 2>&1; then
    pkill -x qs 2>/dev/null || true
    sleep 0.2
    _qs_log="$(paths_cache quickshell)/sidebar-right.log"
    mkdir -p "${_qs_log%/*}"
    qs -c sidebar-right >"$_qs_log" 2>&1 &
  fi
}

case "${1:-}" in
  --startup) RUNTIME=0; NOTIFY=0; REQUESTED="$(read_accent_state)" ;;
  --apply) NOTIFY=0; REQUESTED="$(read_accent_state)" ;;
  --apply-static) RUNTIME=0; NOTIFY=0; REQUESTED="$(read_accent_state)" ;;
  --theme-default)
    RUNTIME=0
    NOTIFY=0
    _active_theme="$(read_state "$ACTIVE_FILE" "$DEFAULT_THEME")"
    if ! REQUESTED="$(theme_default_accent "$_active_theme")"; then
      printf 'No default accent for theme: %s\n' "$_active_theme" >&2
      exit 1
    fi
    ;;
  '') REQUESTED="$(select_accent)"; [ -n "$REQUESTED" ] || exit 0 ;;
  *) REQUESTED="$1" ;;
esac

if [ "${ARGVUS_NO_RUNTIME:-0}" = 1 ]; then
  RUNTIME=0
  NOTIFY=0
fi

if ! normalize_accent "$REQUESTED"; then
  printf 'Invalid accent color: %s\n' "$REQUESTED" >&2
  exit 1
fi

THEME="$(read_state "$ACTIVE_FILE" "$DEFAULT_THEME")"
case "$THEME" in
  argvus-dark|argvus-dark-float|argvus-dark-silver|argvus-dark-silver-float|argvus-light|argvus-light-float|argvus-slate|argvus-slate-float) ;;
  *-dark-float) THEME="argvus-dark-float" ;;
  *-light-float) THEME="argvus-light-float" ;;
  *-light) THEME="argvus-light" ;;
  *) THEME="$DEFAULT_THEME" ;;
esac

mkdir -p "$STATE_DIR"
printf '%s\n' "$THEME" > "$ACTIVE_FILE"
printf '%s\n' "$COLOR" > "$ACCENT_FILE"

apply_theme_references
apply_waybar
apply_rofi
apply_qt_palette
apply_quickshell
apply_application_colors

[ "$RUNTIME" -eq 1 ] && refresh_runtime
if [ "$NOTIFY" -eq 1 ]; then
  notify-send "Accent" "Highlight color: ${COLOR}" 2>/dev/null || true
fi
printf 'Accent %s applied to %s.\n' "$COLOR" "$THEME"
