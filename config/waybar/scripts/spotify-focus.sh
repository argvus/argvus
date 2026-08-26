#!/usr/bin/env sh
# Focus the window of the active music player.

# Map playerctl name → Hyprland window class.
resolve_class() {
    case "$1" in
        spotify*)   echo "spotify" ;;
        audacious*) echo "Audacious" ;;
        rhythmbox*) echo "Rhythmbox" ;;
        strawberry*) echo "Strawberry" ;;
        lollypop*)  echo "Lollypop" ;;
        firefox*)   echo "firefox" ;;
        chrome*)    echo "chrome" ;;
        chromium*)  echo "chromium" ;;
        brave*)     echo "brave" ;;
        edge*)      echo "edge" ;;
        vivaldi*)   echo "vivaldi" ;;
        opera*)     echo "opera" ;;
        *)          echo "$1" ;;
    esac
}

# Native players.
for name in spotify audacious rhythmbox strawberry lollypop; do
    STATUS=$(playerctl -p "$name" status 2>/dev/null)
    if [ "$STATUS" = "Playing" ] || [ "$STATUS" = "Paused" ]; then
        CLASS=$(resolve_class "$name")
        hyprctl dispatch focuswindow "class:$CLASS" 2>/dev/null
        exit 0
    fi
done

# Spotify via browser.
for player in $(playerctl -l 2>/dev/null); do
    case "$(echo "$player" | tr '[:upper:]' '[:lower:]')" in
        spotify|audacious|rhythmbox|strawberry|lollypop) continue ;;
        chrome*|chromium*|firefox*|brave*|edge*|vivaldi*|opera*)
            STATUS=$(playerctl -p "$player" status 2>/dev/null)
            if [ "$STATUS" = "Playing" ] || [ "$STATUS" = "Paused" ]; then
                META=$(playerctl -p "$player" metadata 2>/dev/null)
                case "$META" in
                    *[Ss]potify*)
                        CLASS=$(resolve_class "$player")
                        hyprctl dispatch focuswindow "class:$CLASS" 2>/dev/null
                        exit 0
                        ;;
                esac
            fi
            ;;
    esac
done

exit 0
