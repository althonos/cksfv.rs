#!/bin/sh -e

. $(dirname $0)/functions.sh

# --- Test -------------------------------------------------------------------

log Testing code
cargo test

log Testing code with all features
cargo test --all-features
