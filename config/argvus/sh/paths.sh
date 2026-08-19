# shellcheck shell=sh disable=SC2034

# -- Path helpers -------------------------------------------------------------

ARGVUS_SYSTEM_CONFIG="${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}"
ARGVUS_CONFIG_HOME="${ARGVUS_CONFIG_HOME:-${XDG_CONFIG_HOME:-$HOME/.config}}"
ARGVUS_STATE_HOME="${ARGVUS_STATE_HOME:-${XDG_STATE_HOME:-$HOME/.local/state}/argvus}"
ARGVUS_CACHE_HOME="${ARGVUS_CACHE_HOME:-${XDG_CACHE_HOME:-$HOME/.cache}/argvus}"

paths_user_config() { echo "${ARGVUS_CONFIG_HOME}/${1}"; }
paths_system_config() { echo "${ARGVUS_SYSTEM_CONFIG}/${1}"; }
paths_generated_config() { echo "${ARGVUS_STATE_HOME}/config/${1}"; }
paths_cache() { echo "${ARGVUS_CACHE_HOME}/${1}"; }
paths_state() { echo "${ARGVUS_STATE_HOME}/${1}"; }
paths_argvus_config() { echo "${ARGVUS_CONFIG_HOME}/argvus/${1}"; }
paths_backgrounds() { echo "/usr/share/backgrounds/${1}"; }

paths_read_config() {
  _relative_path="$1"
  _user_path="$(paths_user_config "$_relative_path")"
  _generated_path="$(paths_generated_config "$_relative_path")"
  _system_path="$(paths_system_config "$_relative_path")"

  if [ -e "$_user_path" ] || [ -L "$_user_path" ]; then
    printf '%s\n' "$_user_path"
  elif [ -e "$_generated_path" ] || [ -L "$_generated_path" ]; then
    printf '%s\n' "$_generated_path"
  else
    printf '%s\n' "$_system_path"
  fi
}

paths_ensure_generated_copy() {
  _relative_path="$1"
  _user_path="$(paths_user_config "$_relative_path")"
  _generated_path="$(paths_generated_config "$_relative_path")"
  _system_path="$(paths_system_config "$_relative_path")"
  _source_path="$_system_path"

  if [ -e "$_user_path" ] || [ -L "$_user_path" ]; then
    _source_path="$_user_path"
  fi

  if [ -d "$_source_path" ]; then
    rm -rf "$_generated_path"
    mkdir -p "$_generated_path"
    cp -R "$_source_path/." "$_generated_path/"
  elif [ -f "$_source_path" ]; then
    mkdir -p "${_generated_path%/*}"
    cp "$_source_path" "$_generated_path"
  else
    mkdir -p "${_generated_path%/*}"
  fi

  printf '%s\n' "$_generated_path"
}

paths_config() {
  _relative_path="$1"

  if [ "${ARGVUS_MUTABLE_CONFIG:-0}" = 1 ]; then
    case "$_relative_path" in
      argvus/sh/*|*/scripts/*|*/docs/*)
        paths_read_config "$_relative_path"
        return 0
      ;;
    esac
    paths_ensure_generated_copy "$_relative_path"
    return 0
  fi

  paths_read_config "$_relative_path"
}
