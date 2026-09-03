#!/usr/bin/env sh

set -eu

CDPATH=
ROOT_DIR="$(cd -- "$(dirname -- "$0")/../.." && pwd)"
ORG_DIR="$(cd -- "$ROOT_DIR/.." && pwd)"
MODE="user"
PREFIX="${HOME:-}/.local"
DRY_RUN=false
RESTART=false

usage() {
  cat <<EOF
Usage: tools/sh/install.sh [options]

Installs only the local metapackage metadata from this checkout. Runtime files
are owned by the modular ARGVUS repositories and by the Arch packages pulled by
pacman -S argvus.

Options:
  --user          install metadata to ~/.local (default)
  --system        install metadata to /usr using sudo when needed
  --all           install user and system metadata
  --prefix <dir>  user install prefix (default: ~/.local)
  --copy-config   accepted for compatibility; modular packages own configs
  --force         accepted for compatibility
  --repair        accepted for compatibility
  --restart       reload ARGVUS runtime after install when available
  --dry-run       print actions without changing files
  -h, --help      show this help

For a complete desktop install, use the package repository:

  pacman -S argvus
EOF
}

log() { printf '%s\n' "$*"; }
die() { printf 'install: %s\n' "$*" >&2; exit 1; }

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

needs_sudo() {
  [ "${1#"$HOME"}" = "$1" ] || return 1
  [ "$(id -u)" -ne 0 ] || return 1
  case "$1" in
    /usr|/usr/*|/etc|/etc/*) return 0 ;;
    *) return 1 ;;
  esac
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --user)
      MODE="user"
      shift
    ;;
    --system)
      MODE="system"
      PREFIX="/usr"
      shift
    ;;
    --all)
      MODE="all"
      shift
    ;;
    --prefix)
      [ "$#" -ge 2 ] || die "--prefix requires a value"
      PREFIX="$2"
      shift 2
    ;;
    --copy-config|--force|--repair|--no-setup)
      log "Ignoring $1; modular ARGVUS packages own runtime config files."
      shift
    ;;
    --restart)
      RESTART=true
      shift
    ;;
    --dry-run)
      DRY_RUN=true
      shift
    ;;
    -h|--help)
      usage
      exit 0
    ;;
    *)
      die "unknown argument: $1"
    ;;
  esac
done

[ -n "${HOME:-}" ] || die "HOME is not set"

check_component_checkouts() {
  missing=""
  for repo in \
    argvus-session \
    argvus-app-profiles \
    argvus-appearance \
    argvus-display \
    argvus-lock \
    argvus-network \
    argvus-notifications \
    argvus-portal \
    argvus-power \
    argvus-settings \
    argvus-shell \
    argvus-storage \
    argvus-default-apps \
    argvus-calendar \
    argvus-greeter \
    argvus-waybar
  do
    [ -d "$ORG_DIR/$repo" ] || missing="$missing $repo"
  done

  if [ -n "$missing" ]; then
    log "Missing sibling checkouts:$missing"
    log "This is only a source-tree warning; pacman resolves package dependencies."
  fi
}

install_license() {
  prefix="$1"
  license_path="$prefix/share/licenses/argvus/LICENSE"

  log "Installing argvus metapackage metadata to $prefix..."
  if needs_sudo "$license_path"; then
    sudo_run install -Dm644 "$ROOT_DIR/LICENSE" "$license_path"
  else
    run install -Dm644 "$ROOT_DIR/LICENSE" "$license_path"
  fi
}

restart_runtime() {
  [ "$RESTART" = true ] || return 0

  log "Reloading ARGVUS runtime..."
  if command -v argvus-sessionctl >/dev/null 2>&1; then
    argvus-sessionctl reload >/dev/null 2>&1 || true
  elif command -v hyprctl >/dev/null 2>&1; then
    hyprctl reload >/dev/null 2>&1 || true
  fi
}

check_component_checkouts

case "$MODE" in
  user)
    install_license "$PREFIX"
  ;;
  system)
    install_license /usr
  ;;
  all)
    install_license "${HOME}/.local"
    install_license /usr
  ;;
  *)
    die "invalid mode: $MODE"
  ;;
esac

restart_runtime

log "Metapackage install completed. Runtime files are installed by the ARGVUS modules."
