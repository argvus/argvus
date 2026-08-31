#!/usr/bin/env sh

# Colors using Vivid
if command -v vivid >/dev/null 2>&1; then
  # shellcheck disable=SC2155
  export LS_COLORS="$(vivid generate one-dark)"
fi

command -v argvus >/dev/null 2>&1 && alias spf='argvus --spf'
command -v argvus >/dev/null 2>&1 && alias superfile='argvus --spf'
command -v argvus >/dev/null 2>&1 && alias btop='argvus --btop'
command -v argvus >/dev/null 2>&1 && alias btm='argvus --btm'
command -v argvus >/dev/null 2>&1 && alias bottom='argvus --btm'
command -v argvus >/dev/null 2>&1 && alias yazi='argvus --yazy'
