#!/usr/bin/env bash
set -euo pipefail
owned_pid="$1"
operation="$2"
command -v xclip > /dev/null
command -v xdotool > /dev/null
case "$operation" in
  # A fresh Xvfb desktop has no clipboard owner yet.
  clipboard-read) xclip -selection clipboard -out 2>/dev/null || true; exit 0 ;;
  # xclip forks a selection owner. Close the captured pipes in that child so
  # the test driver can finish waiting for this helper immediately.
  clipboard-write) exec xclip -selection clipboard -in >/dev/null 2>/dev/null ;;
esac
mapfile -t windows < <(xdotool search --onlyvisible --pid "$owned_pid")
if [[ ${#windows[@]} != 1 ]]; then
  echo "Expected one visible window owned by PID $owned_pid, got ${#windows[@]}" >&2
  exit 1
fi
window="${windows[0]}"
# Focus directly: Xvfb need not run a window manager. Do not use --window on
# `key`: that selects XSendEvent instead of the real XTEST server input path.
xdotool windowfocus --sync "$window"
[[ "$(xdotool getwindowfocus)" == "$window" ]]
case "$operation" in
  focus) ;;
  select-all) xdotool key --clearmodifiers ctrl+a ;;
  copy) xdotool key --clearmodifiers ctrl+c ;;
  paste) xdotool key --clearmodifiers ctrl+v ;;
  right) xdotool key --clearmodifiers Right ;;
  *) echo "Unknown input operation $operation" >&2; exit 1 ;;
esac
