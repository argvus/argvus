#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

get_mount_info() {
  mount="$1"

  df -BG "$mount" 2>/dev/null | awk 'NR==2 {
        gsub(/G/, "", $2)
        gsub(/G/, "", $3)
        gsub(/%/, "", $5)

        printf "%s %s %s\n", $3, $2, $5
    }'
}

ROOT_INFO=$(get_mount_info "/")
HOME_INFO=$(get_mount_info "/home")

[ -n "$ROOT_INFO" ] || ROOT_INFO="0 0 0"
[ -n "$HOME_INFO" ] || HOME_INFO="0 0 0"

ROOT_USED=$(printf '%s\n' "$ROOT_INFO" | awk '{print $1}')
ROOT_TOTAL=$(printf '%s\n' "$ROOT_INFO" | awk '{print $2}')
ROOT_PERC=$(printf '%s\n' "$ROOT_INFO" | awk '{print $3}')

HOME_USED=$(printf '%s\n' "$HOME_INFO" | awk '{print $1}')
HOME_TOTAL=$(printf '%s\n' "$HOME_INFO" | awk '{print $2}')
HOME_PERC=$(printf '%s\n' "$HOME_INFO" | awk '{print $3}')

ROOT_BAR=$(string_bar "$ROOT_PERC" "$BAR_SIZE")
HOME_BAR=$(string_bar "$HOME_PERC" "$BAR_SIZE")

TEXT=$(
  printf "%-7s %-14s %-7s %s\n" "Mount" "Used" "Perc" ""
  printf "%-7s %-14s %-7s %s\n" "/" "${ROOT_USED}/${ROOT_TOTAL}G" "${ROOT_PERC}%" "$ROOT_BAR"
  printf "%-7s %-14s %-7s %s\n" "/home" "${HOME_USED}/${HOME_TOTAL}G" "${HOME_PERC}%" "$HOME_BAR"
)

json_output "$TEXT"
