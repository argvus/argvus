#!/usr/bin/env bash
# Tests for the public `argvus` dispatcher. Fake private commands are created
# under /tmp/argvus and removed on exit.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
DISPATCHER="$ROOT_DIR/src/usr/bin/argvus"

failures=0
pass() { printf 'ok - %s\n' "$1"; }
fail() { printf 'not ok - %s\n' "$1" >&2; failures=$((failures + 1)); }

mkdir -p /tmp/argvus
WORK="$(mktemp -d /tmp/argvus/dispatcher.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT

COMMANDS="$WORK/commands"
EMPTY_PATH_DIR="$WORK/empty-path"
PATH_BIN="$WORK/path-bin"
TOOLS="$WORK/tools"
mkdir -p "$COMMANDS" "$EMPTY_PATH_DIR" "$PATH_BIN" "$TOOLS"
# Only the external tools the dispatcher and fakes need. Any ARGVUS command
# installed under /usr/bin stays invisible to the tests.
for tool in cat env; do
  ln -s "$(command -v "$tool")" "$TOOLS/$tool"
done

# Fake private command: prints each argument bracketed (so spaces and empty
# arguments are visible), then reads stdin and exits with $FAKE_STATUS.
make_fake() {
  # shellcheck disable=SC2016
  printf '%s\n' \
    '#!/bin/sh' \
    'for arg do printf "[%s]\n" "$arg"; done' \
    'printf "stdin=%s\n" "$(cat)"' \
    'exit "${FAKE_STATUS:-0}"' > "$1"
  chmod 755 "$1"
}
make_fake "$COMMANDS/argvus-sessionctl"
make_fake "$COMMANDS/argvus-config"

# The PATH is isolated: only the fakes above and the TOOLS links are visible.
# The dispatcher is started through /bin/sh by absolute path for the same reason.
run() {
  env -i PATH="$TOOLS" HOME="$WORK" \
    ARGVUS_COMMANDS_DIR="$COMMANDS" "$@"
}

# 1. No arguments prints usage and succeeds.
out="$(run /bin/sh "$DISPATCHER" 2>&1)" && status=0 || status=$?
if [ "$status" -eq 0 ] && printf '%s' "$out" | command grep -q '^Usage:'; then
  pass "no arguments prints usage"
else
  fail "no arguments prints usage (status $status)"
fi

# 2. help and --help are accepted.
for flag in help --help -h; do
  out="$(run /bin/sh "$DISPATCHER" "$flag" 2>&1)" && status=0 || status=$?
  if [ "$status" -eq 0 ] && printf '%s' "$out" | command grep -q '^Commands:'; then
    pass "$flag prints help"
  else
    fail "$flag prints help (status $status)"
  fi
done

# 3. Unknown command fails with a hint.
out="$(run /bin/sh "$DISPATCHER" does-not-exist 2>&1)" && status=0 || status=$?
if [ "$status" -ne 0 ] && printf '%s' "$out" | command grep -q "argvus --help"; then
  pass "unknown command fails with hint"
else
  fail "unknown command fails with hint (status $status)"
fi

# 4. Arguments are forwarded exactly, including spaces, empty and special ones.
out="$(printf 'in' | run /bin/sh "$DISPATCHER" sessionctl 'a b' '' 'x$y' '*' "it's" 2>&1)" && status=0 || status=$?
# shellcheck disable=SC2016
expected="$(printf '%s\n' '[a b]' '[]' '[x$y]' '[*]' "[it's]" 'stdin=in')"
if [ "$status" -eq 0 ] && [ "$out" = "$expected" ]; then
  pass "arguments are forwarded exactly and stdin is inherited"
else
  fail "arguments are forwarded exactly (status $status, output: $out)"
fi

# 5. The exit status of the private command is returned unchanged.
out="$(run env FAKE_STATUS=7 /bin/sh "$DISPATCHER" config get </dev/null 2>&1)" && status=0 || status=$?
if [ "$status" -eq 7 ]; then
  pass "exit status is preserved"
else
  fail "exit status is preserved (got $status)"
fi

# 6. Works regardless of the current directory.
out="$(cd / && run /bin/sh "$DISPATCHER" sessionctl ok </dev/null 2>&1)" && status=0 || status=$?
if [ "$status" -eq 0 ] && [ "$out" = "$(printf '[ok]\nstdin=')" ]; then
  pass "works outside the project directory"
else
  fail "works outside the project directory (status $status)"
fi

# 7. Commands are resolved from PATH when the commands directory lacks them.
make_fake "$PATH_BIN/argvus-i18n"
out="$(env -i PATH="$PATH_BIN:$TOOLS" HOME="$WORK" \
  ARGVUS_COMMANDS_DIR="$EMPTY_PATH_DIR" /bin/sh "$DISPATCHER" i18n locale </dev/null 2>&1)" && status=0 || status=$?
if [ "$status" -eq 0 ] && [ "$out" = "$(printf '[locale]\nstdin=')" ]; then
  pass "falls back to PATH for accepted names"
else
  fail "falls back to PATH for accepted names (status $status)"
fi

# 8. The commands directory takes precedence over PATH.
make_fake "$PATH_BIN/argvus-config"
out="$(env -i PATH="$PATH_BIN:$TOOLS" HOME="$WORK" \
  ARGVUS_COMMANDS_DIR="$COMMANDS" /bin/sh "$DISPATCHER" config get </dev/null 2>&1)" && status=0 || status=$?
if [ "$status" -eq 0 ] && [ "$out" = "$(printf '[get]\nstdin=')" ]; then
  pass "commands directory wins over PATH"
else
  fail "commands directory wins over PATH (status $status)"
fi

# 9. A known command that is not installed reports it clearly.
out="$(run /bin/sh "$DISPATCHER" firewall status </dev/null 2>&1)" && status=0 || status=$?
if [ "$status" -ne 0 ] && printf '%s' "$out" | command grep -q 'argvus-firewall is not installed'; then
  pass "missing private command is reported"
else
  fail "missing private command is reported (status $status)"
fi

# 10. `argvus` never dispatches to itself, and names outside the table are
# rejected even when a file with that name exists in the commands directory.
make_fake "$COMMANDS/argvus-evil"
out="$(run /bin/sh "$DISPATCHER" argvus status 2>&1)" && status=0 || status=$?
out_evil="$(run /bin/sh "$DISPATCHER" evil 2>&1)" && status_evil=0 || status_evil=$?
if [ "$status" -ne 0 ] && [ "$status_evil" -ne 0 ] && printf '%s' "$out_evil" | command grep -q 'unknown command: evil'; then
  pass "self and unlisted names are not dispatched"
else
  fail "self and unlisted names are not dispatched"
fi

# 11. The `git` subcommand maps to argvus-git and forwards its arguments.
make_fake "$COMMANDS/argvus-git"
out="$(run /bin/sh "$DISPATCHER" git 'my repo' 2>&1 </dev/null)" && status=0 || status=$?
if [ "$status" -eq 0 ] && [ "$out" = "$(printf '[my repo]\nstdin=')" ]; then
  pass "git maps to argvus-git"
else
  fail "git maps to argvus-git (status $status, output: $out)"
fi

if [ "$failures" -ne 0 ]; then
  printf '%s failure(s)\n' "$failures" >&2
  exit 1
fi
printf 'argvus dispatcher tests ok\n'
