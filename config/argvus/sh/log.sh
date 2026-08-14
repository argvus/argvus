# shellcheck shell=sh

# -- Logging utilities --------------------------------------------------------

log_info() { printf "[INFO] %s\n" "$*"; }
log_warn() { printf "[WARN] %s\n" "$*" >&2; }
log_error() { printf "[ERROR] %s\n" "$*" >&2; }
log_die() { log_error "$*"; exit 1; }
