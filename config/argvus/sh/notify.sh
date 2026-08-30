# shellcheck shell=sh

# -- Notification abstraction (notify-send wrapper) ---------------------------

notify_send() {
  summary="$1"
  body="$2"
  notify-send "${summary}" "${body}"
}

notify_error() {
  notify_send "Error: $1" "$2"
}
