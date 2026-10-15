#!/bin/sh
here="$(dirname "$(readlink -f "$0")")"
if command -v unshare >/dev/null 2>&1 && unshare --user --map-root-user true >/dev/null 2>&1; then
  exec "$here/demoflow" "$@"
fi
exec "$here/demoflow" --no-sandbox "$@"
