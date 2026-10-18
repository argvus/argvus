# Waybar Integration

Use this as the clock/date click handler:

```json
"on-click": "/usr/lib/argvus-taskbar-calendar/waybar-launcher --root {x} {y}"
```

The ARGVUS Waybar patch provides `{x}`/`{y}` from the original button event as
immutable root-window coordinates. The launcher passes those desktop
coordinates through `--x`/`--y`; the calendar then resolves the active Waybar
edge and opens immediately below it in both Sticky and Float modes. Waybar
builds without these placeholders retain the live-pointer compatibility
fallback.

The popup aligns its right edge with the horizontal Waybar surface and opens
4 logical pixels below its bottom edge when the bar is at the top, or above
its top edge when the bar is at the bottom. Bottom anchoring follows the actual
popup height, including agenda and settings views. The horizontal bar is
selected on the monitor even when event coordinates fall outside its rectangle;
vertical telemetry panels are excluded. The horizontal bar is recognized by its
layer namespace, either `argvus-taskbar` (ARGVUS taskbar) or `waybar`; the
namespace must match one of these, or the popup falls back to the monitor's
reserved area and opens at the wrong position. Its layer surface uses exclusive zone
`-1`: margins already include the panel position, so the compositor must not
add the panel's reserved area again.
