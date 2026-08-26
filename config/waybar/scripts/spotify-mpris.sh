#!/usr/bin/env sh
# Detect Spotify content — native app or via browser MediaSession.
# Outputs "artist — title" if Spotify is active, empty otherwise.

# 1. Native Spotify app (always shown).
STATUS=$(playerctl -p spotify status 2>/dev/null)
if [ "$STATUS" = "Playing" ] || [ "$STATUS" = "Paused" ]; then
    playerctl -p spotify metadata --format "{{ artist }} — {{ title }}" 2>/dev/null
    exit 0
fi

# 2. Spotify via browser — check each browser player for Spotify metadata.
for player in $(playerctl -l 2>/dev/null); do
    case "$(echo "$player" | tr '[:upper:]' '[:lower:]')" in
        spotify) continue ;;
        chrome*|chromium*|firefox*|brave*|edge*|vivaldi*|opera*)
            STATUS=$(playerctl -p "$player" status 2>/dev/null)
            if [ "$STATUS" = "Playing" ] || [ "$STATUS" = "Paused" ]; then
                META=$(playerctl -p "$player" metadata 2>/dev/null)
                case "$META" in
                    *[Ss]potify*)
                        playerctl -p "$player" metadata --format "{{ artist }} — {{ title }}" 2>/dev/null
                        exit 0
                        ;;
                esac
            fi
            ;;
    esac
done

exit 0
