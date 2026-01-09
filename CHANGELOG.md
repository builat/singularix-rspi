
---

## CHANGELOG.md

```markdown
# Changelog

All notable changes to this project will be documented in this file.

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
