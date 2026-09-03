#!/usr/bin/env sh

set -eu

MODE="user"
DRY_RUN=false

usage() {
  cat <<EOF
Usage: tools/sh/uninstall.sh [--user] [--system] [--all] [--dry-run] [--help]

Removes only metadata installed by this metapackage checkout. Runtime files are
owned by the modular ARGVUS packages and must be removed through pacman or each
module repository.

Options:
  --user          uninstall user metadata from ~/.local (default)
  --system        uninstall system metadata from /usr using sudo
  --all           uninstall both user and system metadata
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

remove_user() {
  log "Removing argvus metapackage user metadata..."
  run rm -f "${HOME}/.local/share/licenses/argvus/LICENSE"
}

remove_system() {
  log "Removing argvus metapackage system metadata..."
  sudo_run rm -f /usr/share/licenses/argvus/LICENSE
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

case "$MODE" in
  user) remove_user ;;
  system) remove_system ;;
  all)
    remove_user
    remove_system
  ;;
  *) die "invalid mode: $MODE" ;;
esac

log "Metapackage uninstall completed."
