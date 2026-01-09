# BLE LED Gateway (Raspberry Pi + Actix) & Web UI

Control addressable LEDs (and an Arduino UNO R4 WiFi) over BLE from a Raspberry Pi, with a modern React UI.  
Features:

- Robust BLE central using **bluer** (scan → stop+settle → connect → `services_resolved()`).
- SSE event stream broadcasting Arduino notifications and server status.
- JSON-encoded ACKs from Arduino’s 3-byte protocol `[status, cmd, detail]`.
- REST API under `/led/*` (color, paint ranges, rainbow).
- Rspack + Mantine dark UI with SSE live log.
- Systemd unit for easy deploy on Raspberry Pi.

## Contents

- `server/` — Actix-web gateway that talks BLE and exposes HTTP + SSE.
- `client/` — React (Rspack) web UI.

---

## Quick start

### 0. Prerequisites (on Raspberry Pi)

```bash
sudo apt update
sudo apt install -y bluetooth bluez bluez-tools pkg-config libdbus-1-dev
sudo systemctl enable --now bluetooth
```

#### Recommended for dev:
In `/etc/bluetooth/main.conf`, set:
```txt
[GATT]
Cache = no
```

### 1. Server (.env)
Create server/.env from the example:
```bash
cp server/.env.example server/.env
```
Edit values:

```bash
WEB_ADDR=0.0.0.0
WEB_PORT=8080
BLE_TARGET_NAME=BleServiceName                          # should be the same as on arduino
BLE_CHUNK_SIZE=20                                       # default value could be changed
BLE_SVC_UUID=f49842a0-def2-4ed6-8636-f48a4332b275       # same as on arduino
BLE_RX_UUID_STR=f4cbb481-052a-4080-8081-2e3f1437b3f3    # - || -
BLE_TX_UUID_STR=6faa3298-1a04-4df0-9348-99492707d883    # - || -
```


### 2. API

All routes are under the /led scope.
```HTTP
GET /led/status -> true | false

POST /led/set-full-color -> { "r":u8, "g":u8, "b":u8 }

POST /led/paint -> { "start":u16, "end":u16, "r":u8, "g":u8, "b":u8 }

POST /led/commands/rainbow -> {} (no body needed)

GET /led/events -> SSE stream of server/Arduino events (JSON lines)

SSE payloads

Arduino 3-byte ACK [status, cmd, detail] becomes JSON:
```
```json
{
  "ts": 1736381000123,
  "status": "ok",
  "command": "paint",
  "detail": "none",
  "status_code": 0,
  "command_code": 3,
  "detail_code": 0
}
```

### Arduino Protocol
```makefile
resp = [status, cmd, detail]
status: 0x00=ok, 0x01=err
detail: 0x00=none, 0x01=unknown, 0x02=bad_args, 0x03=range, 0x04=internal

cmd:    0x01=rainbow, 0x03=paint, 0x04=color_full etc.
```
