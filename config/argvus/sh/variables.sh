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

# -- Application defaults -----------------------------------------------------
# $BOOTSTRAP_DIR is set by bootstrap.sh before sourcing this file. When not
# present (standalone use), fall back to the packaged location on PATH or
# /usr/share/argvus so TERM/FINDER still resolve.
_GET_DEFAULT="${BOOTSTRAP_DIR:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh}/get-default.sh"
get_default_value() {
  _cat="$1"
  if [ -x "$_GET_DEFAULT" ]; then
    sh "$_GET_DEFAULT" "$_cat"
  elif command -v get-default >/dev/null 2>&1; then
    get-default "$_cat"
  else
    # Local fallback matching argvus-default-apps defaults.
    case "$_cat" in
      terminal) printf 'kitty\n' ;;
      file_manager) printf 'spf\n' ;;
      terminal_editor) printf 'vim\n' ;;
      text_editor) printf 'mousepad\n' ;;
      image_viewer) printf 'imv\n' ;;
      pdf_viewer) printf 'zathura\n' ;;
      video_player) printf 'mpv\n' ;;
      audio_player) printf 'audacious\n' ;;
      archive) printf 'xarchiver\n' ;;
      launcher) printf 'rofi\n' ;;
      browser) printf 'xdg-open\n' ;;
      *) printf '\n' ;;
    esac
  fi
}

# -- Application paths --------------------------------------------------------
TERM="$(get_default_value terminal | sed '/^$/d' | head -1)"
[ -n "$TERM" ] || TERM="kitty"
FINDER="$(get_default_value launcher | sed '/^$/d' | head -1)"
[ -n "$FINDER" ] || FINDER="rofi"
FILE_MANAGER="$(get_default_value file_manager | sed '/^$/d' | head -1)"
[ -n "$FILE_MANAGER" ] || FILE_MANAGER="spf"
TERMINAL_EDITOR="$(get_default_value terminal_editor | sed '/^$/d' | head -1)"
[ -n "$TERMINAL_EDITOR" ] || TERMINAL_EDITOR="vim"
TEXT_EDITOR="$(get_default_value text_editor | sed '/^$/d' | head -1)"
[ -n "$TEXT_EDITOR" ] || TEXT_EDITOR="mousepad"
IMAGE_VIEWER="$(get_default_value image_viewer | sed '/^$/d' | head -1)"
[ -n "$IMAGE_VIEWER" ] || IMAGE_VIEWER="imv"
PDF_VIEWER="$(get_default_value pdf_viewer | sed '/^$/d' | head -1)"
[ -n "$PDF_VIEWER" ] || PDF_VIEWER="zathura"
VIDEO_PLAYER="$(get_default_value video_player | sed '/^$/d' | head -1)"
[ -n "$VIDEO_PLAYER" ] || VIDEO_PLAYER="mpv"
AUDIO_PLAYER="$(get_default_value audio_player | sed '/^$/d' | head -1)"
[ -n "$AUDIO_PLAYER" ] || AUDIO_PLAYER="audacious"
ARCHIVE_APP="$(get_default_value archive | sed '/^$/d' | head -1)"
[ -n "$ARCHIVE_APP" ] || ARCHIVE_APP="xarchiver"

# -- UI defaults --------------------------------------------------------------
BAR_SIZE="8"

# -- Active theme -------------------------------------------------------------
ACTIVE_THEME="argvus-dark-aether"
