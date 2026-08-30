#!/usr/bin/env sh
# Toggle play/pause on any supported music player.

# Native players.
for name in spotify audacious rhythmbox strawberry lollypop; do
    STATUS=$(playerctl -p "$name" status 2>/dev/null)
    if [ "$STATUS" = "Playing" ] || [ "$STATUS" = "Paused" ]; then
        playerctl -p "$name" play-pause 2>/dev/null
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
                        playerctl -p "$player" play-pause 2>/dev/null
                        exit 0
                        ;;
                esac
            fi
            ;;
    esac
done

exit 0
