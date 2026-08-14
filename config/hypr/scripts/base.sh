#!/usr/bin/env sh

# shellcheck disable=SC1091
# DEPRECATED: Sourcing base.sh directly is deprecated.
# Prefer: ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

mkdir -p "$HYPR_CACHE_DIR"
