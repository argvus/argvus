#!/usr/bin/env sh

# Check if there is any active MPRIS player (playing or paused).
PLAYER_STATUS=$(playerctl status 2>/dev/null)

if [ "$PLAYER_STATUS" = "Playing" ] || [ "$PLAYER_STATUS" = "Paused" ]; then
  echo ""
  exit 0
fi

# No media playing — shows active window title.
TITLE=$(hyprctl activewindow -j 2>/dev/null | jq -r '.title // empty')

if [ -z "$TITLE" ]; then
  echo ""
  exit 0
fi

# Truncate long title
MAX=60
TITLE_LEN=$(printf '%s' "$TITLE" | wc -m)
if [ "$TITLE_LEN" -gt "$MAX" ]; then
  TITLE=$(printf '%s' "$TITLE" | cut -c1-"$MAX")…
fi

echo " $TITLE"
