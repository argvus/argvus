#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

OS=$(grep '^PRETTY_NAME=' /etc/os-release | cut -d= -f2- | tr -d '"')
LOCALE=$(locale | awk -F= '/^LANG=/{print $2}')
UPTIME=$(uptime -p | sed 's/^up //')
KERNEL=$(uname -r)

CPU=$(LC_ALL=C lscpu | awk -F: '
/Model name/ {
    gsub(/^[ \t]+/, "", $2)
    print $2
    exit
}' | sed 's/(R)//g; s/(TM)//g; s/Core//g; s/  */ /g')

GPU=$(lspci | awk -F': ' '
/VGA compatible controller|3D controller/ {
    print $2
    exit
}' | sed 's/.*\[\(.*\)\].*/\1/')

TEXT=$(
cat <<EOF
<span>OS:</span>       $OS
<span>Kernel:</span>   $KERNEL
<span>Locale:</span>   $LOCALE
<span>Uptime:</span>   $UPTIME
<span>CPU:</span>      $CPU
<span>GPU:</span>      $GPU
EOF
)

json_output "$TEXT"
