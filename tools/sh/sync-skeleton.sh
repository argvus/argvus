#!/bin/sh
# Keeps the shared scaffolding of an ARGVUS repository in step with its skeleton.
#
# .skeleton/manifest    "SHA256  PATH" for every shared file, as the skeleton had it.
# .skeleton/exceptions  "PATH  REASON" for files the repository keeps different on
#                       purpose. Exceptions are never overwritten and not checked.
#
# Usage, from the repository root:
#   sh tools/sh/sync-skeleton.sh --check
#   sh tools/sh/sync-skeleton.sh --apply SKELETON_DIR
set -eu

SHARED_FILES="rust-toolchain.toml rustfmt.toml clippy.toml .editorconfig"
MANIFEST=".skeleton/manifest"
EXCEPTIONS=".skeleton/exceptions"

die() {
  printf 'sync-skeleton: %s\n' "$1" >&2
  exit 1
}

is_excepted() {
  [ -f "$EXCEPTIONS" ] || return 1
  awk -v path="$1" '$1 == path { found = 1 } END { exit !found }' "$EXCEPTIONS"
}

check() {
  [ -f "$MANIFEST" ] || die "missing $MANIFEST; run --apply from the skeleton first"
  drift=0
  while read -r expected path; do
    [ -n "${path:-}" ] || continue
    is_excepted "$path" && continue
    if [ ! -f "$path" ]; then
      printf 'sync-skeleton: missing %s\n' "$path" >&2
      drift=1
      continue
    fi
    actual=$(sha256sum "$path" | awk '{ print $1 }')
    if [ "$actual" != "$expected" ]; then
      printf 'sync-skeleton: %s differs from the skeleton\n' "$path" >&2
      drift=1
    fi
  done < "$MANIFEST"
  [ "$drift" -eq 0 ] || die "shared files drifted; run --apply SKELETON_DIR or record an exception"
  printf 'sync-skeleton: shared files match the skeleton\n'
}

apply() {
  skeleton="$1"
  [ -d "$skeleton" ] || die "skeleton directory not found: $skeleton"
  mkdir -p .skeleton
  : > "$MANIFEST.tmp"
  for path in $SHARED_FILES; do
    is_excepted "$path" && continue
    if [ -f "$skeleton/$path" ]; then
      mkdir -p "$(dirname "$path")"
      cp "$skeleton/$path" "$path"
      printf '%s  %s\n' "$(sha256sum "$path" | awk '{ print $1 }')" "$path" >> "$MANIFEST.tmp"
    fi
  done
  mv "$MANIFEST.tmp" "$MANIFEST"
  printf 'sync-skeleton: applied %s to %s\n' "$skeleton" "$(pwd)"
}

case "${1:-}" in
  --check) check ;;
  --apply) [ "$#" -eq 2 ] || die "usage: --apply SKELETON_DIR"; apply "$2" ;;
  *) die "usage: sync-skeleton.sh --check | --apply SKELETON_DIR" ;;
esac
