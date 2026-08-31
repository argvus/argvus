#!/usr/bin/env sh

# shellcheck disable=SC1090,SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

SUPPRESS_FILE="$(paths_cache hypr)/keyboard-layout-notify.suppress"
mkdir -p "${SUPPRESS_FILE%/*}"
date +%s > "$SUPPRESS_FILE"

cleanup() {
  sleep 1
  rm -f "$SUPPRESS_FILE"
}
trap cleanup EXIT HUP INT TERM

rofimoji \
  --action clipboard \
  --clipboarder wl-copy \
  --typer wtype \
  --selector-args "-config $(paths_config rofi/config.rasi)"
