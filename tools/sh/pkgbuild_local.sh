#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"

PACKAGING_DIR="$ROOT_DIR/packaging/arch/local"

BUILD_SCRIPT="$PACKAGING_DIR/PKGBUILD"
TEMP_BUILD_SCRIPT=""
cleanup() {
  [[ -z "$TEMP_BUILD_SCRIPT" ]] || rm -f "$TEMP_BUILD_SCRIPT"
}
trap cleanup EXIT

metadata="$({ cd "$PACKAGING_DIR" && bash -c 'source "$1"; printf "%s\n%s\n" "$pkgname" "$pkgver"' bash "$BUILD_SCRIPT"; })"
pkgname="$(printf '%s\n' "$metadata" | sed -n '1p')"
pkgver="$(printf '%s\n' "$metadata" | sed -n '2p')"
archive="$ROOT_DIR/build/artifacts/${pkgname}-${pkgver}.tar.gz"
mkdir -p "$ROOT_DIR/build/artifacts"

if grep -q "${pkgname}-\${pkgver}.tar.gz\|\${pkgname}-\${pkgver}.tar.gz\|${pkgname}-${pkgver}.tar.gz" "$BUILD_SCRIPT"; then
  echo "Creating local source archive: $archive"
  tar -czf "$archive" \
    --exclude='./.git' \
    --exclude='./.release' \
    --exclude='./packages-repo' \
    --exclude='./target' \
    --exclude='./build' \
    --exclude='./dist' \
    --exclude='./tmp' \
    --exclude='./tools' \
    --exclude='./packaging/arch/*/src' \
    --exclude='./packaging/arch/*/pkg' \
    --exclude='./packaging/arch/*/*.pkg.tar*' \
    --exclude='./packaging/arch/*/*.tar.gz' \
    --transform "s#^\./#${pkgname}-${pkgver}/#" \
    -C "$ROOT_DIR" .
fi

if [[ -n "${MAKEPKG_FLAGS:-}" ]]; then
  # shellcheck disable=SC2206
  flags=(${MAKEPKG_FLAGS})
elif [[ "$pkgname" == "argvus-waybar" ]]; then
  flags=(--syncdeps --noconfirm --needed --cleanbuild --clean --force)
else
  flags=(--nodeps --noconfirm --needed --cleanbuild --clean --force)
fi

cd "$PACKAGING_DIR"
export BUILDDIR="$ROOT_DIR/build/artifacts"
export SRCDEST="$ROOT_DIR/build/artifacts"
export PKGDEST="$ROOT_DIR/build/dist"
mkdir -p "$PKGDEST"
makepkg -p "$BUILD_SCRIPT" "${flags[@]}" "$@"

packages="$(find "$ROOT_DIR/build/dist" -maxdepth 1 -type f -name "${pkgname}-*.pkg.tar.zst" -print | sort)"
if [[ -n "$packages" ]]; then
  printf 'Packages created:\n%s\n' "$packages"
fi
