#!/usr/bin/env sh

set -eu

MODE="user"
DRY_RUN=false

usage() {
  cat <<EOF
Usage: tools/sh/uninstall.sh [--user] [--system] [--all] [--dry-run] [--help]

Removes Argvus (and legacy archypr-desktop) files installed by
install.sh / make install. Session launchers are owned by the separate
argvus-session project/package. User config directories are preserved
as .bak-* backups and listed at the end so nothing is lost.

Options:
  --user          uninstall user install in ~/.local and ~/.config (default)
  --system        uninstall system install in /usr using sudo
  --all           uninstall both user and system
  --dry-run       print what would be removed without removing
  -h, --help      show this help
EOF
}

log() { printf '%s\n' "$*"; }
die() { printf 'uninstall: %s\n' "$*" >&2; exit 1; }

run() {
  if [ "$DRY_RUN" = true ]; then
    printf '[dry-run] %s\n' "$*"
  else
    "$@"
  fi
}

sudo_run() {
  if [ "$DRY_RUN" = true ]; then
    printf '[dry-run] sudo %s\n' "$*"
  else
    sudo "$@"
  fi
}

rm_if() {
  [ -e "$1" ] || [ -L "$1" ] || return 0
  log "Removing: $1"
  run rm -rf "$1"
}

rm_if_sudo() {
  [ -e "$1" ] || [ -L "$1" ] || return 0
  log "Removing: $1"
  sudo_run rm -rf "$1"
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --user) MODE="user" ;;
    --system) MODE="system" ;;
    --all) MODE="all" ;;
    --dry-run) DRY_RUN=true ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
  shift
done

[ -n "${HOME:-}" ] || die "HOME is not set"

CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
CACHE_HOME="${XDG_CACHE_HOME:-$HOME/.cache}"

uninstall_user() {
  log "Uninstalling Argvus user install..."

  rm_if "$HOME/.local/bin/argvus-setup"
  rm_if "$HOME/.local/bin/argvus-storage"
  rm_if "$HOME/.local/share/argvus"

  # Current centralized layout
  rm_if "$CONFIG_HOME/argvus"
  rm_if "$CONFIG_HOME/argvus-storage"
  rm_if "$CONFIG_HOME/.argvus-bootstrap"
  rm_if "$STATE_HOME/argvus"
  rm_if "$STATE_HOME/argvus-storage"
  rm_if "$CACHE_HOME/argvus"

  # Legacy structure — per-app configs/scripts copied by older argvus-setup or
  # provisioned by the runtime before the centralized ~/.config/argvus layout.
  rm_if "$CONFIG_HOME/argvus-sysinfo"
  rm_if "$CONFIG_HOME/hypr/scripts"
  rm_if "$CONFIG_HOME/hypr/docs"
  rm_if "$CONFIG_HOME/waybar/scripts"
  rm_if "$CONFIG_HOME/kitty/scripts"
  rm_if "$CONFIG_HOME/yazi/scripts"
  rm_if "$CONFIG_HOME/waybar/argvus-taskbar.jsonc"
  rm_if "$CONFIG_HOME/waybar/argvus-taskbar.css"
  rm_if "$CONFIG_HOME/waybar/argvus-sysinfo.jsonc"
  rm_if "$CONFIG_HOME/waybar/argvus-sysinfo.css"

  rm_if "$CONFIG_HOME/environment.d/argvus.conf"

  rm_if "$HOME/.local/bin/archypr-desktop-setup"
  rm_if "$HOME/.local/bin/archypr-desktop-start"
  rm_if "$HOME/.local/bin/archypr-desktop-session"
  rm_if "$HOME/.local/bin/archypr-desktop-tty"
  rm_if "$HOME/.local/bin/archypr-storage"
  rm_if "$HOME/.local/share/archypr-desktop"
  rm_if "$HOME/.local/share/wayland-sessions/archypr-desktop.desktop"
  rm_if "$HOME/.local/share/xsessions/archypr-desktop.desktop"

  rm_if "$CONFIG_HOME/archypr-desktop"
  rm_if "$CONFIG_HOME/.archypr-desktop-bootstrap"

  log "Kept user config backups (newest per app):"
  last_app=""
  for d in "$CONFIG_HOME"/*.bak-*; do
    [ -e "$d" ] || continue
    app="${d##*/}"; app="${app%.bak-*}"
    case "$last_app" in
      "$app") continue ;;
    esac
    last_app="$app"
    newest="$(printf '%s\n' "$CONFIG_HOME"/"$app".bak-* | sort | tail -1)"
    log "  $app -> $newest"
  done
}

uninstall_system() {
  log "Uninstalling Argvus system install..."

  sudo_run rm -f /usr/bin/argvus-setup
  sudo_run rm -f /usr/bin/argvus-storage
  rm_if_sudo /usr/share/argvus
  rm_if_sudo /usr/share/licenses/argvus

  sudo_run rm -f /usr/bin/archypr-desktop-setup
  sudo_run rm -f /usr/bin/archypr-desktop-start
  sudo_run rm -f /usr/bin/archypr-desktop-session
  sudo_run rm -f /usr/bin/archypr-desktop-tty
  sudo_run rm -f /usr/bin/archypr-storage
  rm_if_sudo /usr/share/archypr-desktop
  rm_if_sudo /usr/share/wayland-sessions/archypr-desktop.desktop
  rm_if_sudo /usr/share/xsessions/archypr-desktop.desktop

  sudo_run rm -f /etc/profile.d/argvus.sh
  sudo_run rm -f /etc/environment.d/argvus.conf
}

case "$MODE" in
  user) uninstall_user ;;
  system) uninstall_system ;;
  all)
    uninstall_user
    uninstall_system
  ;;
  *) die "invalid mode: $MODE" ;;
esac

log "Uninstall completed."
