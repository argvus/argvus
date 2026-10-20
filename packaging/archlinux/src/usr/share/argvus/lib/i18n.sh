# shellcheck shell=sh
ARGVUS_COMMANDS_DIR="${ARGVUS_COMMANDS_DIR:-/usr/lib/argvus/commands}"

# Login-path scripts source this library before the user PATH is complete, so
# the packaged dispatcher is preferred by its absolute path. Other scripts fall
# back to the dispatcher on PATH.
argvus_i18n_run() {
    if [ -x /usr/bin/argvus ]; then
        /usr/bin/argvus i18n "$@"
    else
        argvus i18n "$@"
    fi
}

argvus_tr() {
    if [ "$#" -lt 2 ]; then
        printf '%s\n' 'argvus_tr: expected DOMAIN KEY' >&2
        return 2
    fi
    argvus_i18n_run get "$@"
}

argvus_locale() {
    argvus_i18n_run locale
}
