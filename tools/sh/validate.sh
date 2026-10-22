#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "$SCRIPT_DIR/../.." && pwd)"

die() { printf 'error: %s\n' "$1" >&2; exit 1; }
require_command() { command -v "$1" >/dev/null 2>&1 || die "required command not found: $1"; }

require_command bash
require_command makepkg
require_command shellcheck

cd "$ROOT_DIR"
# Every shell script the package installs, including the greeter launchers that
# run before login and as root (greeter-setup).
packaged_scripts=(
	packaging/archlinux/src/usr/bin/argvus-terminal
	packaging/archlinux/src/usr/lib/argvus/taskbar-calendar/waybar-launcher
	packaging/archlinux/src/usr/share/argvus/lib/i18n.sh
	packaging/archlinux/src/usr/share/argvus/*/sh/*.sh
	crates/argvus-greeter/packaging/greetd/argvus-greeter-session
	crates/argvus-greeter/packaging/greetd/argvus-greeter-setup
	crates/argvus-greeter/packaging/greetd/argvus-greeter-tui
)
shellcheck tools/sh/*.sh packaging/archlinux/pkgbuilds/common/*.sh "${packaged_scripts[@]}"
for script in "${packaged_scripts[@]}"; do
	sh -n "$script"
done

# Packaged tmpfiles entries cross a privilege boundary (the greeter directory is
# shared with the session that logs in): none may be writable by every user.
for tmpfiles in crates/argvus-greeter/packaging/greetd/*tmpfiles*.conf; do
	if awk '!/^[[:space:]]*(#|$)/ && $3 ~ /^[0-7]*[2367]$/ { found = 1 } END { exit !found }' "$tmpfiles"; then
		die "world-writable tmpfiles entry in $tmpfiles"
	fi
done
bash -n tools/sh/*.sh packaging/archlinux/pkgbuilds/common/*.sh

metadata() {
	bash -c '
		source "$1"
		printf "%s\n" "$pkgbase" "$pkgver" "$pkgrel" "$pkgdesc"
		printf "%s\n" "${arch[*]}" "${license[*]}" "${depends[*]}"
		printf "%s\n" "${makedepends[*]}" "${options[*]}"
	' bash "$1"
}

remote_metadata="$(metadata packaging/archlinux/pkgbuilds/remote/PKGBUILD)"
local_metadata="$(metadata packaging/archlinux/pkgbuilds/local/PKGBUILD)"
[[ "$remote_metadata" == "$local_metadata" ]] || die "remote and local PKGBUILD metadata are out of sync"

for pkgbuild_dir in packaging/archlinux/pkgbuilds/remote packaging/archlinux/pkgbuilds/local; do
	(
		cd "$pkgbuild_dir"
		makepkg -p PKGBUILD --printsrcinfo >/dev/null
	)
done

git diff --check
printf 'argvus validation ok\n'
