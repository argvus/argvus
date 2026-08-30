#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

get_default_iface() {
  ip route 2>/dev/null |
    awk '/default/ {print $5; exit}'
}

IFACE="$(get_default_iface)"

if [ -z "$IFACE" ]; then
  printf '{"text":"Disconnected"}\n'
  exit 0
fi

IP_ADDR=$(
  ip -4 addr show "$IFACE" 2>/dev/null |
    awk '/inet / {print $2}' |
    cut -d/ -f1 |
    head -n1
)

RX1=$(cat "/sys/class/net/$IFACE/statistics/rx_bytes" 2>/dev/null)
TX1=$(cat "/sys/class/net/$IFACE/statistics/tx_bytes" 2>/dev/null)

RX1=${RX1:-0}
TX1=${TX1:-0}

sleep 1

RX2=$(cat "/sys/class/net/$IFACE/statistics/rx_bytes" 2>/dev/null)
TX2=$(cat "/sys/class/net/$IFACE/statistics/tx_bytes" 2>/dev/null)

RX2=${RX2:-$RX1}
TX2=${TX2:-$TX1}

RX_RATE=$((RX2 - RX1))
TX_RATE=$((TX2 - TX1))

human() {
  awk -v b="$1" '
    BEGIN {
        if (b >= 1073741824)
            printf "%.1fGiB/s", b/1073741824
        else if (b >= 1048576)
            printf "%.1fMiB/s", b/1048576
        else if (b >= 1024)
            printf "%.1fKiB/s", b/1024
        else
            printf "%dB/s", b
    }'
}

RX_H=$(human "$RX_RATE")
TX_H=$(human "$TX_RATE")

TEXT=$(
  cat <<EOF
IFACE  IP              Traffic
$IFACE  $IP_ADDR  ↓ $RX_H ↑ $TX_H
EOF
)

json_output "$TEXT"
