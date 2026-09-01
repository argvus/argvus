#!/usr/bin/env sh

set -eu

CDPATH=
ROOT_DIR="$(cd -- "$(dirname -- "$0")/../.." && pwd)"
MODE="user"
SETUP_MODE="copy-all"
DRY_RUN=false
COPY_CONFIG=false
RESTART=false
PREFIX="${HOME:-}/.local"
STORAGE_DIR="${ARGVUS_STORAGE_DIR:-$(cd -- "$ROOT_DIR/../argvus-storage" && pwd)}"
STORAGE_RESOURCES="$STORAGE_DIR/resources"

usage() {
  cat <<EOF
Usage: tools/sh/install.sh [options]

Installs the current argvus checkout for local testing, together
with the sibling argvus-storage checkout (../argvus-storage), without
requiring a GitHub release or package repository. Session launchers are
owned by the separate argvus-session project/package.

Options:
  --user          install to ~/.local (default)
  --system        install to /usr and /etc using sudo, like the Arch package
  --all           install to both ~/.local and /usr (system + user)
  --prefix <dir>  user install prefix (default: ~/.local)
  --copy-config   explicitly copy packaged defaults into $XDG_CONFIG_HOME
  --force         with --copy-config, replace configs after backups
  --repair        backward-compatible alias for --copy-config --copy argvus
  --restart       restart Waybar/Argvus user services after install
  --dry-run       print setup actions where supported
  -h, --help      show this help

Examples:
  make install
  tools/sh/install.sh --user --restart
  tools/sh/install.sh --system
  tools/sh/install.sh --all
EOF
}

log() { printf '%s\n' "$*"; }
die() { printf 'install: %s\n' "$*" >&2; exit 1; }

need() {
  command -v "$1" >/dev/null 2>&1 || die "missing command: $1"
}

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
    --force)
      SETUP_MODE="copy-all-force"
      shift
    ;;
    --repair)
      SETUP_MODE="repair"
      COPY_CONFIG=true
      shift
    ;;
    --copy-config)
      COPY_CONFIG=true
      shift
    ;;
    --no-setup)
      COPY_CONFIG=false
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
need cargo
need install

[ -f "$STORAGE_DIR/Cargo.toml" ] || die "argvus-storage checkout not found at: $STORAGE_DIR (clone it next to argvus)"

build_storage() {
  log "Building argvus-storage..."
  if [ "$DRY_RUN" = true ]; then
    printf '[dry-run] cargo build --release --locked --manifest-path %s/Cargo.toml\n' "$STORAGE_DIR"
  else
    (cd "$STORAGE_DIR" && cargo build --release --locked)
  fi
}

run_setup() {
  [ "$COPY_CONFIG" = true ] || return 0

  setup_arg=""
  case "$SETUP_MODE" in
    copy-all) setup_arg="--copy-all" ;;
    copy-all-force) setup_arg="--copy-all --force" ;;
    repair) setup_arg="--repair" ;;
  esac

  run_argvus_setup() {
    config_src="$1"
    setup_bin="$2"

    if [ "$DRY_RUN" = true ]; then
      # shellcheck disable=SC2086
      ARGVUS_CONFIG_SRC="$config_src" "$setup_bin" --setup $setup_arg --dry-run
    else
      # shellcheck disable=SC2086
      ARGVUS_CONFIG_SRC="$config_src" "$setup_bin" --setup $setup_arg
    fi
  }

  case "$MODE" in
    system)
      log "Applying system configuration with argvus --setup $setup_arg..."
      # shellcheck disable=SC2086
      run_argvus_setup "/usr/share/argvus" argvus
    ;;
    user|all)
      log "Applying user configuration with argvus --setup $setup_arg..."
      # shellcheck disable=SC2086
      run_argvus_setup "$ROOT_DIR/config" "$ROOT_DIR/bin/argvus"

      run mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/argvus-storage"
      run cp "$STORAGE_RESOURCES/config.json" \
        "${XDG_CONFIG_HOME:-$HOME/.config}/argvus-storage/config.json"
      log "Installed user storage config: ${XDG_CONFIG_HOME:-$HOME/.config}/argvus-storage/config.json"
      run cp "$STORAGE_RESOURCES/theme.css" \
        "${XDG_CONFIG_HOME:-$HOME/.config}/argvus-storage/theme.css"
      log "Installed user storage theme: ${XDG_CONFIG_HOME:-$HOME/.config}/argvus-storage/theme.css"
      if [ -d "$STORAGE_RESOURCES/themes" ]; then
        run mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/argvus-storage/themes"
        run cp -R "$STORAGE_RESOURCES/themes/." \
          "${XDG_CONFIG_HOME:-$HOME/.config}/argvus-storage/themes/"
      fi
    ;;
  esac
}

install_user() {
  bin_dir="$PREFIX/bin"
  share_dir="$PREFIX/share/argvus"

  log "Installing Argvus to $PREFIX..."
  run mkdir -p "$bin_dir" "$share_dir"

  run rm -rf "$share_dir"
  run cp -R "$ROOT_DIR/config/." "$share_dir/"
  run install -m 755 "$ROOT_DIR/bin/argvus" "$bin_dir/argvus"
  run install -m 755 "$STORAGE_DIR/target/release/argvus-storage" "$bin_dir/argvus-storage"

  log "Installed binaries to: $bin_dir"
  log "Make sure $bin_dir is in PATH before restarting the session."

  install_envd_user
}

install_envd_user() {
  # $XDG_CONFIG_HOME/environment.d is read by the systemd user manager, so
  # services started at login get the source config search path + Qt theme
  # without needing a --copy-all materialization at install time. This is a
  # single small file, not a full per-user copy of the DE.
  envd_dir="${XDG_CONFIG_HOME:-$HOME/.config}/environment.d"
  run mkdir -p "$envd_dir"
  run cp "$ROOT_DIR/config/environment.d/argvus.conf" "$envd_dir/argvus.conf"
  log "Installed user environment: $envd_dir/argvus.conf"
}

install_system() {
  need sudo
  log "Installing Argvus to /usr with sudo..."

  sudo_run install -dm755 /usr/share/argvus
  if [ "$DRY_RUN" = true ]; then
    printf '[dry-run] sudo rm -rf /usr/share/argvus\n'
    printf '[dry-run] sudo cp -a %s/config/. /usr/share/argvus/\n' "$ROOT_DIR"
  else
    sudo rm -rf /usr/share/argvus
    sudo cp -a "$ROOT_DIR/config/." /usr/share/argvus/
  fi

  sudo_run install -Dm755 "$ROOT_DIR/bin/argvus" /usr/bin/argvus
  sudo_run install -Dm755 "$STORAGE_DIR/target/release/argvus-storage" /usr/bin/argvus-storage
  sudo_run rm -f /etc/profile.d/argvus.sh
  log "Removed legacy TTY auto-start profile: /etc/profile.d/argvus.sh"
  sudo_run install -Dm644 "$STORAGE_RESOURCES/config.json" \
    /etc/argvus-storage/config.json
  sudo_run install -Dm644 "$STORAGE_RESOURCES/theme.css" \
    /etc/argvus-storage/theme.css
  if [ -d "$STORAGE_RESOURCES/themes" ]; then
    sudo_run install -dm755 /etc/argvus-storage/themes
    if [ "$DRY_RUN" = true ]; then
      printf '[dry-run] sudo cp -a %s/resources/themes/. /etc/argvus-storage/themes/\n' "$STORAGE_DIR"
    else
      sudo cp -a "$STORAGE_RESOURCES/themes/." /etc/argvus-storage/themes/
    fi
  fi
  sudo_run install -Dm644 "$ROOT_DIR/LICENSE" \
    /usr/share/licenses/LICENSE

  install_envd_system
}

install_envd_system() {
  # Ship systemd.environment.d defaults for the systemd user manager so that
  # services started at login (hyprpolkitagent, xdg-desktop-portal, ...) get
  # the Argvus config search path + Qt theme without any per-user ~/.config
  # copy. Applied for NEW logins only (systemd --user reload or re-login).
  sudo_run install -Dm644 "$ROOT_DIR/config/environment.d/argvus.conf" \
    /etc/environment.d/argvus.conf
  log "Installed systemd environment: /etc/environment.d/argvus.conf"
}

restart_runtime() {
  [ "$RESTART" = true ] || return 0

  log "Restarting runtime pieces..."
  if command -v argvus-sessionctl >/dev/null 2>&1; then
    argvus-sessionctl reload >/dev/null 2>&1 || true
    return 0
  fi
  if command -v hyprctl >/dev/null 2>&1; then
    hyprctl reload >/dev/null 2>&1 || true
  fi
}

build_storage

case "$MODE" in
  user) install_user ;;
  system) install_system ;;
  all)
    install_user
    install_system
  ;;
  *) die "invalid mode: $MODE" ;;
esac

run_setup
restart_runtime

log "Install completed."
