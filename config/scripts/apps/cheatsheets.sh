#!/usr/bin/env sh

# Show the cheatsheet for an app (hypr or kitty) through the configured
# launcher. Usage: cheatsheets.sh [hypr|kitty]

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

APP="${1:-hypr}"

if locale_is_pt; then
  CHEAT_FILE="$(paths_config "$APP/docs/cheatsheets/pt.txt")"
  PROMPT="Procurar"
else
  CHEAT_FILE="$(paths_config "$APP/docs/cheatsheets/en.txt")"
  PROMPT="Search"
fi

# FINDER may be a bare name (rofi/wofi) or an absolute path; compare by base
# name so launcher changes from the default-apps state keep working.
_finder_base="${FINDER##*/}"
_finder_base="${_finder_base%% *}"

case "$_finder_base" in
  rofi)
    rofi -dmenu -p "$PROMPT" -i -theme-str 'window { width: 1050px; height: 600px;}' < "$CHEAT_FILE"
    ;;
  wofi)
    wofi --show dmenu < "$CHEAT_FILE"
    ;;
  fuzzel)
    fuzzel --dmenu --prompt "$PROMPT" < "$CHEAT_FILE"
    ;;
  *)
    # Generic launcher: most menus accept the stdin-as-menu pattern.
    "$_finder_base" < "$CHEAT_FILE"
    ;;
esac
