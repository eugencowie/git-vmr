#!/usr/bin/env sh
#
# Run rustfmt from the nightly toolchain, which rustfmt.toml requires.
#
# mise.toml points RUSTFMT here, so `cargo fmt` (and editors which call it)
# use nightly while everything else stays on stable.
#

exec mise exec rust@nightly -- rustfmt "$@"
