# shellcheck shell=sh

# -- String utilities ---------------------------------------------------------

# Draw a progress bar: string_bar <percent> <size>
string_bar() {
  percent=$1
  size=$2

  case "$percent" in
    ""|*[!0-9]*) percent=0 ;;
  esac

  filled=$((percent * size / 100))
  empty=$((size - filled))

  i=0
  while [ "$i" -lt "$filled" ]; do
    printf "█"
    i=$((i + 1))
  done

  i=0
  while [ "$i" -lt "$empty" ]; do
    printf "░"
    i=$((i + 1))
  done
}
