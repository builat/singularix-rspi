# Singularix RSPi — LED control server (Raspberry Pi + Actix)

Controls a WS2812-family LED strip directly from a Raspberry Pi 4. All LED
logic (solid colors, ranges, brightness, the moving rainbow) runs in this
server; no Arduino / BLE link is involved anymore. The strip's data wire is
driven by the Pi's **SPI hardware** (MOSI = GPIO 10, pin 19), so the
timing-critical WS2812 signal is generated in hardware and is immune to
system load — no real-time tricks, no root required.

Features:

- Direct WS2812 output via `/dev/spidev0.0` (800 kHz protocol encoded on a
  3.2 MHz SPI clock, 4 SPI bits per LED bit).
- Port of the old Arduino firmware behavior: command semantics, error ACKs,
  gamma-corrected rainbow (45 ms frames), global brightness, range painting.
- SSE event stream (`GET /led/events`) broadcasting per-command JSON ACKs,
  same shape the Arduino's 3-byte BLE notifications used to produce.
- REST API under `/led/*`.
- Systemd unit for deploy on the Pi (`deploy/singularix-rspi.service`).

## Wiring

| Strip wire | Connect to |
|---|---|
| Data (was on Arduino D6) | **GPIO 10 / MOSI, physical pin 19** |
| Ground | Pi GND, common with the strip's 5 V supply ground |
| 5 V | external power supply (do not power 274 LEDs from the Pi) |

Notes:

- The Pi's data pin is 3.3 V logic while the strip runs at 5 V. This usually
  works; if colors flicker, add a level shifter (e.g. 74AHCT125) on the data
  line.
- GPIO 17 / any plain GPIO **cannot** generate the WS2812 timing on Linux;
  the wire must be on the SPI MOSI pin.
- A ~1000 µF capacitor across the strip's 5 V input is recommended.

## Setup (on the Pi)

1. Enable SPI and give your user access:

   ```bash
   sudo raspi-config          # Interface Options -> SPI -> enable, then reboot
   ls /dev/spidev0.0          # must exist
   sudo usermod -aG spi,gpio pi
   ```

2. Build and configure:

   ```bash
   cargo build --release
   cp .env_example .env       # adjust LED_COUNT / LED_COLOR_ORDER if needed
   ```

3. Run it (or install the systemd unit):

   ```bash
   sudo cp deploy/singularix-rspi.service /etc/systemd/system/
   sudo systemctl enable --now singularix-rspi
   ```

## Configuration (`.env`)

```bash
WEB_ADDR=0.0.0.0                 # web server bind address
WEB_PORT=8080                    # web server port
LED_COUNT=274                    # LEDs on the strip (firmware used 274)
LED_COLOR_ORDER=brg              # wire byte order: rgb/rbg/grb/gbr/brg/bgr
LED_SPI_DEVICE=/dev/spidev0.0    # SPI device (MOSI = GPIO 10)
LED_SPI_SPEED_HZ=3200000         # 4 x the 800 kHz WS2812 bit rate
```

All `LED_*` values are optional and default to the values above. If colors
look swapped, try a different `LED_COLOR_ORDER` (most strips are `grb`).

## API

All routes are under the `/led` scope. Responses are plain text; every
command also produces a JSON ACK on the SSE stream.

```HTTP
POST /led/color      -> { "r": i32, "g": i32, "b": i32 }   # whole strip
POST /led/range      -> { "start": u16, "end": u16, "r": u8, "g": u8, "b": u8 }
POST /led/brightness -> 0..255 (bare JSON number)
POST /led/rainbow    -> no body
POST /led/off        -> no body
POST /led/debug      -> raw firmware command line, e.g. `4 255 0 0`
GET  /led/events     -> SSE stream of ACK events
GET  /ble-status     -> `true` while the LED engine runs (legacy route)
```

`/led/debug` accepts the firmware's ASCII grammar: `0` (off), `1` (rainbow),
`3 <start> <end> <r> <g> <b>`, `4 <r> <g> <b>`, `5 <brightness>`, `99` (noop).

Firmware behaviors kept 1:1:

- Any recognized command stops the rainbow (even a failing one); only `1`
  restarts it. Unparseable input or ids outside 0..=99 do not touch it.
- Range indices are swapped/clamped, never rejected; RGB and brightness
  values are strictly validated (0..=255).
- Rainbow: 45 ms frames, +180 hue per frame, gamma-corrected, resumes from
  where it stopped.
- Brightness is a global scale over the raw colors: `0` goes dark but the
  colors are remembered.

### SSE payloads

Each command is ACKed as JSON (same shape as the old Arduino notifications):

```json
{
  "ts": 1736381000123,
  "status": "ok",
  "command": "set rgb",
  "detail": "none",
  "status_code": 0,
  "command_code": 4,
  "detail_code": 0
}
```

```makefile
resp = [status, cmd, detail]
status: 0x00=ok, 0x01=err
detail: 0x00=none, 0x01=unknown, 0x02=bad_args, 0x03=range, 0x04=internal
cmd:    0x00=off, 0x01=rainbow, 0x03=set range, 0x04=set rgb, 0x05=brightness, 0x11=switch off
```

## Development

`cargo test` and `cargo check` work on any machine — the SPI device code is
compiled only on Linux; elsewhere the server runs with a no-op LED sink.
