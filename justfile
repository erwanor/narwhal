#!/usr/bin/env -S just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

build:
    cargo build

format:
    cargo +nightly fmt --all

lint:
    just format
    cargo clippy --all -- -D clippy::all -D warnings -D clippy::disallowed_method

license:
    ./scripts/license_check.sh

test:
  cargo nextest run --features celo --profile ci


prepare-pr:
    just test
    just lint
    just check-license
