#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

EDITOR=nvim
YAZI="/usr/bin/yazi"
ZATHURA="/usr/bin/zathura"

target=$1

[ -z "$target" ] && exit 1

if [ -d "$target" ]; then
    "$TERM" -e "$YAZI" "$target" >/dev/null 2>&1 &
    exit 0
fi

mime=$(file -Lb --mime-type "$target")

case "$mime" in
    text/*|application/json|\
    inode/x-empty|\
    application/xml|\
    application/toml|\
    application/x-yaml|\
    application/x-shellscript)
        "$TERM" -e "$EDITOR" "$target" >/dev/null 2>&1 &
        ;;
    application/pdf)
        "$ZATHURA" "$target" >/dev/null 2>&1 &
        ;;
    image/*)
        imv "$target" >/dev/null 2>&1 &
        ;;
    video/*)
        mpv "$target" >/dev/null 2>&1 &
        ;;
    audio/*)
        mpv "$target" >/dev/null 2>&1 &
        ;;
    application/zip|\
    application/x-7z-compressed|\
    application/x-rar|\
    application/x-tar|\
    application/gzip|\
    application/x-bzip2|\
    application/x-xz)
        file-roller "$target" >/dev/null 2>&1 &
        ;;
    *)
        xdg-open "$target" >/dev/null 2>&1 &
        ;;
esac
exit 0
