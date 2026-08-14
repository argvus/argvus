# shellcheck shell=sh disable=SC2034

# -- Environment root ----------------------------------------------------------
ARGVUS_SYSTEM_CONFIG="${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}"
ARGVUS_CONFIG_HOME="${ARGVUS_CONFIG_HOME:-${XDG_CONFIG_HOME:-$HOME/.config}}"
ARGVUS_STATE_HOME="${ARGVUS_STATE_HOME:-${XDG_STATE_HOME:-$HOME/.local/state}/argvus}"
ARGVUS_CACHE_HOME="${ARGVUS_CACHE_HOME:-${XDG_CACHE_HOME:-$HOME/.cache}/argvus}"
ENVIRONMENT_ROOT="${ENVIRONMENT_ROOT:-$ARGVUS_SYSTEM_CONFIG/argvus}"

# -- Cache directories --------------------------------------------------------
HYPR_CACHE_DIR="${ARGVUS_CACHE_HOME}/hypr"
WAYBAR_CACHE_DIR="${ARGVUS_CACHE_HOME}/waybar"

# -- Set variables global ------------------------------------------------------
# Button Layout restored usage: appmenu:minimize,maximize,close
BUTTON_LAYOUT=":"
GTK_THEME="Adwaita-dark"
ICON_THEME="Yaru-prussiangreen-dark"
GTK_CURSOR="Adwaita"

# -- Application paths --------------------------------------------------------
FINDER="/usr/bin/rofi"
TERM="/usr/bin/kitty"

# -- UI defaults --------------------------------------------------------------
BAR_SIZE="8"

# -- Active theme -------------------------------------------------------------
ACTIVE_THEME="argvus-dark"
