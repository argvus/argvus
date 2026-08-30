#!/usr/bin/env sh
# shellcheck shell=sh disable=SC1091

# =============================================================================
# bootstrap.sh — Carrega automaticamente todos os módulos compartilhados.
#
# Uso em scripts:
#   . "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh"
#
# Isso disponibiliza todas as APIs (log_*, string_*, json_*, ...)
# e variáveis globais (WALLPAPER_PATH, BUTTON_LAYOUT, ...).
# =============================================================================

: "${HOME:?HOME is not set}"

ARGVUS_SYSTEM_CONFIG="${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}"
ARGVUS_CONFIG_HOME="${ARGVUS_CONFIG_HOME:-${XDG_CONFIG_HOME:-$HOME/.config}}"
ARGVUS_STATE_HOME="${ARGVUS_STATE_HOME:-${ARGVUS_CONFIG_HOME}/argvus/state}"
ARGVUS_CACHE_HOME="${ARGVUS_CACHE_HOME:-${XDG_CACHE_HOME:-$HOME/.cache}/argvus}"
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-$ARGVUS_SYSTEM_CONFIG/scripts/argvus/bootstrap.sh}"
case ":${XDG_CONFIG_DIRS:-/etc/xdg}:" in
  *":$ARGVUS_SYSTEM_CONFIG:"*) ;;
  *) XDG_CONFIG_DIRS="$ARGVUS_SYSTEM_CONFIG:${XDG_CONFIG_DIRS:-/etc/xdg}" ;;
esac
export XDG_CONFIG_DIRS

BOOTSTRAP_DIR="$(CDPATH= cd -- "$(dirname -- "$ARGVUS_BOOTSTRAP")" && pwd)"
MODULES_DIR="$BOOTSTRAP_DIR"

. "${MODULES_DIR}/variables.sh"
. "${MODULES_DIR}/paths.sh"
. "${MODULES_DIR}/locale.sh"
. "${MODULES_DIR}/log.sh"
. "${MODULES_DIR}/notify.sh"
. "${MODULES_DIR}/string.sh"
. "${MODULES_DIR}/json.sh"
. "${MODULES_DIR}/hypr.sh"
