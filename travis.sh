#!/bin/bash

cd "$DIR/uavcan"
cargo clean
if ["${NO_DEFAULT_FEATURES}"]; then
    cargo test --no-default-features
else
    cargo test
fi
