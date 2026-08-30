#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

GPU_SCRIPT="$(paths_config "waybar/scripts/sysinfo/gpu.sh")"

cpu_temp() {
  # Prioritizes known CPU sensors
  for h in /sys/class/hwmon/hwmon*; do
    [ -r "$h/name" ] || continue

    case "$(cat "$h/name")" in
    coretemp | k10temp | zenpower)
      for label in "$h"/temp*_label; do
        [ -r "$label" ] || continue

        case "$(cat "$label")" in
        "Package id 0" | Tctl | Tdie | Package)
          input="${label%_label}_input"
          awk '{printf "%.0f", $1/1000}' "$input"
          return
          ;;
        esac
      done

      awk '{printf "%.0f", $1/1000}' "$h/temp1_input" 2>/dev/null
      return
      ;;
    esac
  done

  # Fallback
  for t in /sys/class/hwmon/hwmon*/temp*_input; do
    [ -r "$t" ] || continue
    awk '{printf "%.0f", $1/1000}' "$t"
    return
  done

  echo "N/A"
}

cpu_usage() {
  awk '
    BEGIN {
        while ((getline < "/proc/stat") > 0) {
            if ($1 == "cpu") {
                idle1=$5
                total1=0
                for(i=2;i<=NF;i++) total1+=$i
                break
            }
        }
        close("/proc/stat")

        system("sleep 0.2")

        while ((getline < "/proc/stat") > 0) {
            if ($1 == "cpu") {
                idle2=$5
                total2=0
                for(i=2;i<=NF;i++) total2+=$i
                break
            }
        }

        usage=int((1-((idle2-idle1)/(total2-total1)))*100)
        print usage
    }'
}

CPU_TEMP="$(cpu_temp)°C"
CPU_USAGE="$(cpu_usage)%"

if [ -x "$GPU_SCRIPT" ]; then
  GPU_TEMP="$("$GPU_SCRIPT" --temp)"
  GPU_USAGE="$("$GPU_SCRIPT" --usage)"
else
  GPU_TEMP="N/A"
  GPU_USAGE="N/A"
fi

CPU_BAR=$(string_bar "${CPU_USAGE%\%}" "$BAR_SIZE")

TEXT=$(
  printf "%-12s %-7s %-6s %s\n" "Device" "Temp" "Use" ""
  printf "%-12s %-7s %-6s %s\n" "CPU" "$CPU_TEMP" "$CPU_USAGE" "$CPU_BAR"
  printf "%-12s %-7s %-6s\n" "GPU" "$GPU_TEMP" "$GPU_USAGE"
)

json_output "$TEXT"
