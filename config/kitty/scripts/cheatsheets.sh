#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

if locale_is_pt; then
  CHEAT_FILE="$(paths_config kitty/docs/cheatsheets/pt.txt)"
  PROMPT="Procurar"
else
  CHEAT_FILE="$(paths_config kitty/docs/cheatsheets/en.txt)"
  PROMPT="Search"
fi

if [ "$FINDER" = "/usr/bin/rofi" ]; then
  cat "$CHEAT_FILE" | $FINDER -dmenu -p "$PROMPT" -i -theme-str 'window { height: 600px;}'
elif [ "$FINDER" = "/usr/bin/wofi" ]; then
  cat "$CHEAT_FILE" | $FINDER
fi
