
---
## CHANGELOG.md

```markdown
# Changelog

All notable changes to this project will be documented in this file.

## 2026-10-16
### Changed
- Arduino removed: the LED strip (WS2812) is driven directly by the server
  over the Pi's SPI hardware (MOSI = GPIO 10, pin 19, /dev/spidev0.0).
- All firmware logic ported into the server: solid/range colors, brightness,
  off, and the gamma-corrected moving rainbow (45 ms frames, +180 hue/frame).
- Command semantics, validation quirks, and SSE ACK JSON kept 1:1 with the
  firmware; ACKs are now synthesized by the LED engine.
- New `led` module: `protocol` (command parsing/ACKs), `color` (Adafruit
  ColorHSV/gamma port), `device` (SPI encoder + null device), `engine`
  (render thread owning all LED state).

### Added
- `POST /led/off` (firmware command 0 previously had no HTTP route).
- Systemd unit `deploy/singularix-rspi.service` (runs without root; needs
  the `spi` group).
- Unit tests for the protocol quirks, color math, SPI encoding, and engine
  state machine.
- Raspberry Pi cross-compile target: `scripts/build-pi.sh` produces the
  aarch64 Linux binary (glibc 2.17 baseline) from any host using zig cc
  and llvm-ar; toolchain pinned in `rust-toolchain.toml`, linker/CC/AR
  wired in `.cargo/config.toml`. Optional `PI_HOST` deploys over scp.
- Startup check that the SPI frame fits the spidev transfer buffer
  (4096 bytes by default; strips above ~339 LEDs need `spidev.bufsiz`
  raised) with an actionable error instead of per-render failures.

### Removed
- BLE backend (`bluer`, `libdbus-sys`), UUID/chunk env settings, and the
  BLE reconnect logic. New env settings: `LED_COUNT`, `LED_COLOR_ORDER`,
  `LED_SPI_DEVICE`, `LED_SPI_SPEED_HZ` (all defaulted).

### Breaking
- The data wire must move from the Arduino to GPIO 10 (pin 19); SPI must be
  enabled on the Pi.
- Intentional behavior differences: effects keep running with no client
  connected; brightness scales at render time (lossless) instead of the
  library's in-place buffer rescale; ACK names added for `brightness` (5)
  and `off` (0).

## 2026-01-09
### Added
- SSE broadcasting of BLE events; 15s keep-alives.
- JSON parsing for Arduino 3-byte ACKs.
- `/led/*` scoped endpoints (status, set-full-color, paint, rainbow, events).

### Changed
- More robust BLE connect (stop scan, settle, wait services, cache purge, backoff).

### Fixed
- Reduced intermittent `le-connection-abort-by-local`.

### Breaking
- API moved under `/led/*`.
```
