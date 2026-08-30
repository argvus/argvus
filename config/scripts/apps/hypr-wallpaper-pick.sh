#!/usr/bin/env sh

ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"
ARGVUS_MUTABLE_CONFIG=1
HYPRPAPER_FILE="$(paths_config hypr/hyprpaper.conf)"

WALLPAPERS_DIR="/usr/share/backgrounds/argvus"
SELECTED_FILE=$(mktemp)

apply_wallpaper_runtime() {
  _wall="$1"
  hypr_apply_wallpaper "$_wall"
}

get_active_monitor() {
  if command -v hyprctl >/dev/null 2>&1; then
    hyprctl monitors 2>/dev/null |
      sed -n 's/^Monitor \([^ ]*\).*/\1/p' |
      head -n1
  fi
}

"$TERM" -e yazi --chooser-file="$SELECTED_FILE" "$WALLPAPERS_DIR"

SELECTED_PATH=$(cat "$SELECTED_FILE")
rm -f "$SELECTED_FILE"

[ -z "$SELECTED_PATH" ] && exit 0

# Convert $HOME to ~ for config file consistency
CONFIG_PATH=$(echo "$SELECTED_PATH" | sed "s|^$HOME|~|")
MONITOR="$(get_active_monitor)"

# Update hyprpaper.conf with ~ path
[ -n "$MONITOR" ] && sed -i "s|^[[:space:]]*monitor[[:space:]]*=.*$|  monitor = ${MONITOR}|" "$HYPRPAPER_FILE"
sed -i "s|^[[:space:]]*path[[:space:]]*=.*$|  path =  ${CONFIG_PATH}|" "$HYPRPAPER_FILE"

# Apply with full path
apply_wallpaper_runtime "$SELECTED_PATH"

# Rebuild Hyprlock config and invalidate the cached lock wallpaper.
sh "$(paths_config scripts/argvus/hyprlock-theme.sh)" --invalidate >/dev/null 2>&1 || true

notify-send "Wallpaper" "Alterado para:\n$(basename "$SELECTED_PATH")"
