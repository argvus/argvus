# shellcheck shell=sh

# -- JSON utilities for Waybar output -----------------------------------------

json_escape() {
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

json_output() {
  printf '{"text":"%s"}\n' \
    "$(printf '%s' "$1" | sed ':a;N;$!ba;s/\n/\\n/g')"
}
