#!/bin/sh
#
# Cross-compiles singularix-rspi for the Raspberry Pi 4 (aarch64 Linux),
# glibc 2.17 baseline — runs on every Raspberry Pi OS release.
#
# Requirements:
#   - zig            (brew install zig)          — compiles/links C and the binary
#   - llvm-ar        (brew install llvm)         — archives the C library
#   - rustup with the stable toolchain           — provides the aarch64 target std
#
# Usage:
#   scripts/build-pi.sh                 # build only
#   PI_HOST=pi@raspberrypi scripts/build-pi.sh
#                                       # also copy to /opt/singularix-rspi
#                                       # on the Pi and restart the service
#
set -eu
cd "$(dirname "$0")/.."

TARGET=aarch64-unknown-linux-gnu

command -v zig >/dev/null 2>&1 || {
    echo "error: zig not found — brew install zig" >&2
    exit 1
}

# Route cargo through the rustup-managed toolchain so the aarch64 target
# std is available even when the cargo on PATH is a different install
# (e.g. Homebrew rust); its sysroot would be missing the target.
CARGO=""
if command -v rustup >/dev/null 2>&1; then
    rustup target add "$TARGET" >/dev/null 2>&1 || true
    CARGO=$(rustup which cargo) || CARGO=""
fi
[ -n "$CARGO" ] || CARGO=cargo
TC_DIR=$(dirname "$CARGO")

# The toolchain cargo may pick up a foreign rustc from PATH; pin it.
RUSTC="$TC_DIR/rustc" "$CARGO" build --release --target "$TARGET"

BIN="target/$TARGET/release/singularix-rspi"
echo "built: $BIN"
file "$BIN" | sed 's/^/  /'

if [ -n "${PI_HOST:-}" ]; then
    echo "deploying to ${PI_HOST}:/opt/singularix-rspi ..."
    scp "$BIN" "$PI_HOST:/opt/singularix-rspi/singularix-rspi"
    ssh "$PI_HOST" "sudo systemctl restart singularix-rspi && systemctl is-active singularix-rspi"
    echo "deployed and restarted."
fi
