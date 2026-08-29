#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

# Default apps resolve from the argvus-default-apps state (via variables.sh),
# falling back to the Argvus built-ins.
EDITOR="${TERMINAL_EDITOR:-nvim}"
TEXT_EDITOR="${TEXT_EDITOR:-nvim}"
YAZI="$FILE_MANAGER"
[ -n "$YAZI" ] || YAZI="/usr/bin/yazi"
ZATHURA="${PDF_VIEWER:-zathura}"
IMAGE_VIEWER="${IMAGE_VIEWER:-imv}"
VIDEO_PLAYER="${VIDEO_PLAYER:-mpv}"
AUDIO_PLAYER="${AUDIO_PLAYER:-mpv}"
ARCHIVE_APP="${ARCHIVE_APP:-file-roller}"

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
        "$IMAGE_VIEWER" "$target" >/dev/null 2>&1 &
        ;;
    video/*)
        "$VIDEO_PLAYER" "$target" >/dev/null 2>&1 &
        ;;
    audio/*)
        "$AUDIO_PLAYER" "$target" >/dev/null 2>&1 &
        ;;
    application/zip|\
    application/x-7z-compressed|\
    application/x-rar|\
    application/x-tar|\
    application/gzip|\
    application/x-bzip2|\
    application/x-xz)
        "$ARCHIVE_APP" "$target" >/dev/null 2>&1 &
        ;;
    *)
        xdg-open "$target" >/dev/null 2>&1 &
        ;;
esac
exit 0
