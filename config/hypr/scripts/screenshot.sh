#!/usr/bin/env sh

# shellcheck disable=SC1091
ARGVUS_BOOTSTRAP="${ARGVUS_BOOTSTRAP:-${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/bootstrap.sh}"
. "$ARGVUS_BOOTSTRAP"

PICTURES=$(xdg-user-dir PICTURES)
VIDEOS=$(xdg-user-dir VIDEOS)

case "$LANG" in
  pt_*)
    FOLDER_IMAGES="Capturas de tela"
    FILENAME_IMAGE="Captura de tela de"
    FOLDER_VIDEOS="Gravações de tela"
    FILENAME_VIDEO="Gravação de tela de"
    MSG_RECORDING_STARTED="Gravação iniciada"
    MSG_RECORDING_PAUSED="Gravação pausada"
    MSG_RECORDING_RESUMED="Gravação retomada"
    MSG_RECORDING_STOPED="Gravação salva"
    MSG_NO_RECORDING="Sem gravação"
    ;;
  *)
    FOLDER_IMAGES="Screenshots"
    FILENAME_IMAGE="Screenshot"
    FOLDER_VIDEOS="Screen recordings"
    FILENAME_VIDEO="Screen recording"
    MSG_RECORDING_STARTED="Recording started"
    MSG_RECORDING_PAUSED="Recording paused"
    MSG_RECORDING_RESUMED="Recording resumed"
    MSG_RECORDING_STOPED="Recording saved"
    MSG_NO_RECORDING="No recording"
    ;;
esac

mkdir -p "$PICTURES/$FOLDER_IMAGES" "$VIDEOS/$FOLDER_VIDEOS"

PATH_IMAGES="$PICTURES/$FOLDER_IMAGES"
FILENAME_IMAGE_DATETIME="$FILENAME_IMAGE $(date +%Y-%m-%d_%H-%M-%S).png"
PATH_VIDEOS="$VIDEOS/$FOLDER_VIDEOS"
FILENAME_VIDEO_DATETIME="$FILENAME_VIDEO $(date +%Y-%m-%d_%H-%M-%S).mkv"
PID_FILE="$HYPR_CACHE_DIR/gpu-screen-recorder.pid"
STATE_FILE="$HYPR_CACHE_DIR/gpu-screen-recorder.state"

for OLD_NAME in "Capturas de tela" "Screenshots"; do
  [ "$OLD_NAME" = "$FOLDER_IMAGES" ] && continue
  OLD_DIR="$PICTURES/$OLD_NAME"
  [ -d "$OLD_DIR" ] || continue
  TARGET_DIR="$PICTURES/$FOLDER_IMAGES"
  if [ ! -d "$TARGET_DIR" ]; then
    mv "$OLD_DIR" "$TARGET_DIR"
  else
    mv "$OLD_DIR"/* "$TARGET_DIR"/ 2>/dev/null
    rmdir "$OLD_DIR" 2>/dev/null
  fi
done

for OLD_NAME in "Gravações de tela" "Screen recordings"; do
  [ "$OLD_NAME" = "$FOLDER_VIDEOS" ] && continue
  OLD_DIR="$VIDEOS/$OLD_NAME"
  [ -d "$OLD_DIR" ] || continue
  TARGET_DIR="$VIDEOS/$FOLDER_VIDEOS"
  if [ ! -d "$TARGET_DIR" ]; then
    mv "$OLD_DIR" "$TARGET_DIR"
  else
    mv "$OLD_DIR"/* "$TARGET_DIR"/ 2>/dev/null
    rmdir "$OLD_DIR" 2>/dev/null
  fi
done

case "$1" in
  # Options image
  --image-region)
    hyprshot -m region -o "$PATH_IMAGES" -f "$FILENAME_IMAGE_DATETIME"
    satty --filename "$PATH_IMAGES/$FILENAME_IMAGE_DATETIME"
    ;;

  --image-full)
    hyprshot -m output -o "$PATH_IMAGES" -f "$FILENAME_IMAGE_DATETIME"
    ;;

  --image-window)
    hyprshot -m window -o "$PATH_IMAGES" -f "$FILENAME_IMAGE_DATETIME"
    ;;

  # Options video
  --video-full)
    if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
      STATE=$(cat "$STATE_FILE" 2>/dev/null)

      # Recording paused
      if [ "$STATE" = "recording" ]; then
        pkill -SIGUSR2 -f "^gpu-screen-recorder"
        echo paused > "$STATE_FILE"
        notify-send "$FILENAME_VIDEO_DATETIME" "$MSG_RECORDING_PAUSED"

      # Recording resumed
      elif [ "$STATE" = "paused" ]; then
        pkill -SIGUSR2 -f "^gpu-screen-recorder"
        echo recording > "$STATE_FILE"
        notify-send "$FILENAME_VIDEO_DATETIME" "$MSG_RECORDING_RESUMED"
      fi

    # Recording started
    else
      gpu-screen-recorder \
        -w screen \
        -f 60 \
        -a default_output \
        -a default_input \
        -o "$PATH_VIDEOS/$FILENAME_VIDEO_DATETIME" &

      echo $! > "$PID_FILE"
      echo recording > "$STATE_FILE"
      notify-send "$FILENAME_VIDEO_DATETIME" "$MSG_RECORDING_STARTED"
    fi
    ;;

  --video-full-stop)
    if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
      pkill -SIGINT -f "^gpu-screen-recorder"
      rm -f "$PID_FILE" "$STATE_FILE"
      notify-send "$FILENAME_VIDEO_DATETIME" "$MSG_RECORDING_STOPED"
    else
      rm -f "$PID_FILE" "$STATE_FILE"
      notify-send "$FILENAME_VIDEO_DATETIME" "$MSG_NO_RECORDING"
    fi
    ;;
--video-full-status)
  if [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null; then
    STATE=$(cat "$STATE_FILE" 2>/dev/null)

    if [ "$STATE" = "paused" ]; then
      echo "{\"text\":\"\uf04c\",\"tooltip\":\"$MSG_RECORDING_PAUSED\",\"class\":\"paused\"}"
    else
      echo "{\"text\":\"\uf03d\",\"tooltip\":\"$MSG_RECORDING_STARTED\",\"class\":\"recording\"}"
    fi
  else
    echo "{\"text\":\"\uf4e2\",\"tooltip\":\"$MSG_NO_RECORDING\",\"class\":\"stopped\"}"
  fi
  ;;
esac
