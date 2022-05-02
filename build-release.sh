#!/usr/bin/env bash
set -e
cd "$(dirname -- "${BASH_SOURCE[0]}")"

FEATURES="$1"
if [[ -z "$FEATURES" ]]
then
    FEATURES="$XDG_SESSION_TYPE"
fi

RUSTFLAGS="-C link-arg=-s" cargo build --release --features="$FEATURES"
cp target/release/flyandshoot ./
