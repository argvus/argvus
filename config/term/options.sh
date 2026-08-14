#!/usr/bin/env sh

# Colors using Vivid
if command -v vivid >/dev/null 2>&1; then
  # shellcheck disable=SC2155
  export LS_COLORS="$(vivid generate one-dark)"
fi
