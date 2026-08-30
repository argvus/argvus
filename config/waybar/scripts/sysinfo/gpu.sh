#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

# ==============================================================================
# gpu.sh — GPU usage and temperature for Waybar
# Compatible with: NVIDIA (proprietary driver), NVIDIA (nouveau), AMD (amdgpu/radeon)
#
# Use in waybar (config):
#   "custom/gpu": {
#       "exec": "~/.config/waybar/scripts/sysinfo/gpu.sh",
#       "return-type": "json",
#       "interval": 3,
#       "tooltip": true
#   }
#
# JSON: {"text":"...", "tooltip":"...", "class":"..."}
# Classes CSS: gpu-normal | gpu-warning | gpu-critical | gpu-unknown
# ==============================================================================

# GPU driver/manufacturer detection
# Order of priority: nvidia-smi → lsmod → /sys vendor id
# ------------------------------------------------------------------------------
detect_gpu() {
  # nvidia-smi available → NVIDIA proprietary driver
  if command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi -L >/dev/null 2>&1; then
    echo "nvidia_proprietary"
    return
  fi

  # nouveau module loaded
  if lsmod 2>/dev/null | grep -q "^nouveau[[:space:]]"; then
    echo "nvidia_nouveau"
    return
  fi

  # AMDGPU module loaded
  if lsmod 2>/dev/null | grep -q "^amdgpu[[:space:]]"; then
    echo "amd"
    return
  fi

  # radeon (AMD legacy) loaded
  if lsmod 2>/dev/null | grep -q "^radeon[[:space:]]"; then
    echo "amd_radeon"
    return
  fi

  # Fallback: read vendor id in /sys/class/drm
  for vendor_file in /sys/class/drm/card*/device/vendor; do
    [ -r "$vendor_file" ] || continue
    vendor=$(cat "$vendor_file" 2>/dev/null)
    case "$vendor" in
    0x10de)
      echo "nvidia_nouveau"
      return
      ;;
    0x1002)
      echo "amd"
      return
      ;;
    esac
  done

  echo "unknown"
}

# NVIDIA — proprietary driver (nvidia-smi)
# ------------------------------------------------------------------------------
get_nvidia_proprietary() {
  # Get usage% and temperature of GPU 0 (first GPU)
  line=$(nvidia-smi \
    --query-gpu=utilization.gpu,temperature.gpu,name \
    --format=csv,noheader,nounits 2>/dev/null | head -1)

  if [ -z "$line" ]; then
    echo "N/A N/A NVIDIA"
    return
  fi

  usage=$(echo "$line" | awk -F',' '{gsub(/[[:space:]]/,"",$1); print $1}')
  temp=$(echo "$line" | awk -F',' '{gsub(/[[:space:]]/,"",$2); print $2}')
  name=$(echo "$line" | awk -F',' '{gsub(/^[[:space:]]+/,"",$3); print $3}')

  echo "$usage $temp $name"
}

# NVIDIA — driver nouveau (open-source)
# Temperature via hwmon; usage is not reliably exposed by the driver
# ------------------------------------------------------------------------------
get_nvidia_nouveau() {
  usage="N/A"
  temp="N/A"
  name="NVIDIA (nouveau)"

  # Temperature: travels through hwmons looking for "nouveau"
  for hwmon in /sys/class/hwmon/hwmon*; do
    [ -r "$hwmon/name" ] || continue
    hwmon_name=$(cat "$hwmon/name" 2>/dev/null)
    [ "$hwmon_name" = "nouveau" ] || continue

    # Try temp1_input (milli-Celsius)
    for t_file in "$hwmon/temp1_input" "$hwmon/temp2_input"; do
      [ -r "$t_file" ] || continue
      raw=$(cat "$t_file" 2>/dev/null)
      temp=$((raw / 1000))
      break
    done
    break
  done

  # Usage: attempts via debugfs (requires video permission or group)
  debug_clients="/sys/kernel/debug/dri/0/clients"
  if [ -r "$debug_clients" ]; then
    # There is no direct metric; leave it as N/A.
    :
  fi

  # Actual GPU name via /sys
  for card in /sys/class/drm/card*/device/vendor; do
    [ -r "$card" ] || continue
    v=$(cat "$card" 2>/dev/null)
    [ "$v" = "0x10de" ] || continue
    model_file="${card%/vendor}/product_name"
    if [ -r "$model_file" ]; then
      name="NVIDIA $(cat "$model_file" 2>/dev/null) (nouveau)"
    fi
    break
  done

  echo "$usage $temp $name"
}

# AMD — amdgpu / radeon
# Use: /sys/class/drm/card*/device/gpu_busy_percent
# Temperature: hwmon (amdgpu/radeon)
# ------------------------------------------------------------------------------
get_amd() {
  usage="N/A"
  temp="N/A"
  name="AMD GPU"

  # --- GPU Usage ---
  for card_dir in /sys/class/drm/card*/device; do
    busy="$card_dir/gpu_busy_percent"
    [ -r "$busy" ] || continue

    # Confirm it's an AMD product using the vendor ID.
    vendor_file="$card_dir/vendor"
    if [ -r "$vendor_file" ]; then
      v=$(cat "$vendor_file" 2>/dev/null)
      [ "$v" = "0x1002" ] || continue
    fi

    usage=$(cat "$busy" 2>/dev/null)
    break
  done

  # --- Temperature via hwmon ---
  for hwmon in /sys/class/hwmon/hwmon*; do
    [ -r "$hwmon/name" ] || continue
    hwmon_name=$(cat "$hwmon/name" 2>/dev/null)
    case "$hwmon_name" in
    amdgpu | radeon) : ;;
    *) continue ;;
    esac

    # Preference: temp2 (junction/chip) > temp1 (edge/sensor)
    if [ -r "$hwmon/temp2_input" ]; then
      raw=$(cat "$hwmon/temp2_input" 2>/dev/null)
    elif [ -r "$hwmon/temp1_input" ]; then
      raw=$(cat "$hwmon/temp1_input" 2>/dev/null)
    else
      continue
    fi
    temp=$((raw / 1000))
    break
  done

  # --- Model name ---
  for card_dir in /sys/class/drm/card*/device; do
    vendor_file="$card_dir/vendor"
    [ -r "$vendor_file" ] || continue
    v=$(cat "$vendor_file" 2>/dev/null)
    [ "$v" = "0x1002" ] || continue

    # Try product_name first, then generic label.
    for f in "$card_dir/product_name" "$card_dir/label"; do
      if [ -r "$f" ]; then
        model=$(cat "$f" 2>/dev/null)
        [ -n "$model" ] && name="AMD $model"
        break
      fi
    done
    break
  done

  echo "$usage $temp $name"
}

# Temperature-based CSS class
# ------------------------------------------------------------------------------
get_class() {
  t=$1
  if [ "$t" = "N/A" ]; then
    echo "gpu-unknown"
  elif [ "$t" -ge 85 ] 2>/dev/null; then
    echo "gpu-critical"
  elif [ "$t" -ge 70 ] 2>/dev/null; then
    echo "gpu-warning"
  else
    echo "gpu-normal"
  fi
}

# ==============================================================================
# MAIN
# ==============================================================================
GPU_TYPE=$(detect_gpu)

case "$GPU_TYPE" in
nvidia_proprietary)
  data=$(get_nvidia_proprietary)
  icon="󰢮"
  ;;
nvidia_nouveau)
  data=$(get_nvidia_nouveau)
  icon="󰢮"
  ;;
amd | amd_radeon)
  data=$(get_amd)
  icon="󰾲"
  ;;
*)
  case "$1" in
  --temp) echo "N/A" ;;
  --usage) echo "N/A" ;;
  *)
    printf '{"text":"󰾲 GPU N/A","tooltip":"GPU não detectada","class":"gpu-unknown"}\n'
    ;;
  esac
  exit 0
  ;;
esac

usage=$(printf '%s\n' "$data" | awk '{print $1}')
temp=$(printf '%s\n' "$data" | awk '{print $2}')
name=$(printf '%s\n' "$data" | cut -d' ' -f3-)

[ "$usage" != "N/A" ] && usage_str="${usage}%" || usage_str="N/A"
[ "$temp" != "N/A" ] && temp_str="${temp}°C" || temp_str="N/A"

case "$1" in
--temp)
  echo "$temp_str"
  exit 0
  ;;

--usage)
  echo "$usage_str"
  exit 0
  ;;

--name)
  echo "$name"
  exit 0
  ;;
esac

css_class=$(get_class "$temp")

case "$GPU_TYPE" in
nvidia_proprietary) driver_label="NVIDIA (proprietário)" ;;
nvidia_nouveau) driver_label="NVIDIA (nouveau)" ;;
amd) driver_label="AMD (amdgpu)" ;;
amd_radeon) driver_label="AMD (radeon)" ;;
*) driver_label="Desconhecido" ;;
esac

text_val="${icon} ${usage_str}  ${temp_str}"
tooltip_val="$(json_escape "$name") | Driver: $driver_label | Uso: $usage_str | Temp: $temp_str"

printf '{"text":"%s","tooltip":"%s","class":"%s"}\n' \
  "$(json_escape "$text_val")" \
  "$tooltip_val" \
  "$css_class"
