#!/bin/sh

. $(dirname $0)/functions.sh

# --- Setup cargo-cache ------------------------------------------------------

LATEST=$(cargo search cargo-cache | head -n1 | cut -f2 -d"\"")
LOCAL=$(cargo cache --version 2>/dev/null | cut -d" " -f2 || echo "none")

if [ "$LATEST" != "$LOCAL" ]; then
  log Installing cargo-cache v$LATEST
  cargo install -f cargo-cache --no-default-features --features ci-autoclean --root "$HOME/.cargo"
else
  log Using cached cargo-cache v$LOCAL
fi
