#!/usr/bin/env bash


export DIFF=0
# export UPDATE=1
export SWC_CHECK=0
export SWC_RUN=0

touch tests/compress.rs

RUST_LOG=off UPDATE=1 cargo test -p swc_ecma_minifier -p swc --test size
cargo test -p swc_ecma_minifier -p swc --no-fail-fast --test projects --test tsc --test compress --test mangle --features concurrent $@

# find ../swc/tests/ -type f -empty -delete