#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

N_PROC="5"
PS="/usr/bin/ps"
AWK="/usr/bin/awk"

# shellcheck disable=SC2016
OUTPUT=$(
  {
    printf "%-7s %-5s %-5s %s\n" "PID" "CPU%" "MEM%" "CMD"

    $PS -eo pid,pcpu,pmem,comm --no-headers |
      sort -k2 -rn |
      head -n "$N_PROC" |
      $AWK '
        {
            printf "%-7s %-5.1f %-5.1f %s\n",
                   $1, $2, $3, $4
        }'
  }
)

json_output "$OUTPUT"
