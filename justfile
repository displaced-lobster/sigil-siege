# just manual: https://github.com/casey/just/#readme

_default:
    @just --list

# Runs clippy
check:
    cargo clippy --locked -- -D warnings

# Runs rustfmt
fmt:
    cargo +nightly fmt
