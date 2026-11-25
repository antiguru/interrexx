#!/bin/sh
# Builds every target and runs the `standalone` nextest profile on a 32-bit x86 userland.
#
# Run as root inside an i386 Debian container with the checkout mounted (the `i686` job of
# `.github/workflows/test.yml`), so that the toolchain's host, the build scripts, the C and C++
# fixtures the tests compile and load, and the test binaries are all 32-bit, as on a 32-bit
# system. It installs the packages the tests run, then builds and tests as the owner of the
# checkout: some tests rely on limits and permissions that root is exempt from. `RUSTUP_HOME`,
# `CARGO_HOME` and `CARGO_TARGET_DIR` name directories of the mount the job caches.
set -eu

NEXTEST_VERSION=0.9.150

if [ "$(id -u)" = 0 ]; then
    apt-get update
    apt-get install -y --no-install-recommends ca-certificates curl g++ gcc git libc6-dev make \
        python3 subversion time
    owner=$(stat -c %u .)
    group=$(stat -c %g .)
    mkdir -p "$RUSTUP_HOME" "$CARGO_HOME" "$CARGO_TARGET_DIR" /tmp/home
    chown -R "$owner:$group" "$RUSTUP_HOME" "$CARGO_HOME" "$CARGO_TARGET_DIR" /tmp/home
    exec setpriv --reuid="$owner" --regid="$group" --clear-groups env HOME=/tmp/home "$0"
fi

if ! test -x "$CARGO_HOME/bin/rustup"; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs |
        sh -s -- -y --no-modify-path --profile minimal --default-toolchain none \
            --default-host i686-unknown-linux-gnu
fi
export PATH="$CARGO_HOME/bin:$PATH"

cd rust
# `rust-toolchain.toml` names the channel; rustup installs it for this host on first use.
rustc -vV
rustc -vV | grep -qx 'host: i686-unknown-linux-gnu'

if ! cargo nextest --version 2>/dev/null | grep -q "^cargo-nextest $NEXTEST_VERSION"; then
    cargo install cargo-nextest --version "$NEXTEST_VERSION" --locked
fi

cargo build --locked --workspace --all-targets
cargo nextest run --locked --workspace --profile standalone
