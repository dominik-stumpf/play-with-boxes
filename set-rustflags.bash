#!/bin/bash

RUSTFLAGS="-C debug_assertions=false"
export RUSTFLAGS
echo "RUSTFLAGS variable exported with value: $RUSTFLAGS"
