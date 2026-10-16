/*!
    Port of the ASCII command protocol spoken by the singularix-ble-led
    Arduino firmware (src/main.ino, processCommand). Semantics and error
    codes are kept 1:1 so /led/debug and the SSE ACK stream behave like
    the BLE setup did.
*/

use serde::Serialize;

// ACK triple: [status, cmd echo, detail]
pub const ST_OK: u8 = 0x00;
pub const ST_ERR: u8 = 0x01;

pub const D_NONE: u8 = 0x00;
pub const D_UNKNOWN: u8 = 0x01;
pub const D_BAD_ARGS: u8 = 0x02;
pub const D_RANGE: u8 = 0x03;
pub const D_INTERNAL: u8 = 0x04;

pub const CMD_UNPARSED: u8 = 0xFF;

// Command ids echoed in ACKs.
pub const CMD_OFF_RAW: u8 = 0x00; // debug input `0`
pub const CMD_RAINBOW: u8 = 0x01;
pub const CMD_RANGE: u8 = 0x03;
pub const CMD_COLOR: u8 = 0x04;
pub const CMD_BRIGHTNESS: u8 = 0x05;
pub const CMD_NOOP: u8 = 99;
pub const CMD_OFF: u8 = 0x11; // switch off (named code from the protocol table)

/// A validated command the engine can apply. Range indices are kept raw
/// (possibly reversed / out of bounds): like the Arduino's setRangeColor,
/// the engine swaps and clamps them at apply time instead of rejecting.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LedCommand {
    Off,
    Rainbow,
    Range {
        start: i32,
        end: i32,
        r: u8,
        g: u8,
        b: u8,
    },
    Color {
        r: u8,
        g: u8,
        b: u8,
    },
    Brightness {
        value: u8,
    },
    Noop,
}

/// Result of parsing one command line: what to ACK, what to apply, and
/// whether the rainbow animation gets stopped (on the Arduino every command
/// recognized past the id bounds-check stops it — even a failing one; only
/// `1` restarts it).
#[derive(Debug, Clone)]
pub struct Parsed {
    pub status: u8,
    pub cmd_code: u8,
    pub detail: u8,
    pub command: Option<LedCommand>,
    pub stops_rainbow: bool,
}

impl Parsed {
    fn ok(cmd_code: u8, command: LedCommand) -> Self {
        Parsed {
            status: ST_OK,
            cmd_code,
            detail: D_NONE,
            command: Some(command),
            stops_rainbow: true,
        }
    }

    fn err(cmd_code: u8, detail: u8, stops_rainbow: bool) -> Self {
        Parsed {
            status: ST_ERR,
            cmd_code,
            detail,
            command: None,
            stops_rainbow,
        }
    }
}

/// Parses a raw command line, e.g. `4 255 0 0` or `3 10 20 128 128 128`.
///
/// Mirrors the firmware's `sscanf("%d %d %d %d %d %d")`: up to six
/// whitespace-separated integers; parsing stops at the first token that is
/// not a plain integer; extra arguments beyond the sixth are ignored.
pub fn parse_ascii(input: &str) -> Parsed {
    let mut vals = [0i32; 6];
    let mut count = 0usize;
    for (slot, token) in vals.iter_mut().zip(input.split_whitespace()) {
        match token.parse::<i32>() {
            Ok(v) => {
                *slot = v;
                count += 1;
            }
            Err(_) => break,
        }
    }

    if count < 1 {
        return Parsed::err(CMD_UNPARSED, D_BAD_ARGS, false);
    }
    let cmd = vals[0];
    if !(-0..=99).contains(&cmd) {
        return Parsed::err(CMD_UNPARSED, D_RANGE, false);
    }

    match cmd {
        0 => Parsed::ok(CMD_OFF_RAW, LedCommand::Off),
        1 => Parsed::ok(CMD_RAINBOW, LedCommand::Rainbow),
        3 => {
            if count < 6 {
                return Parsed::err(CMD_RANGE, D_BAD_ARGS, true);
            }
            match rgb_checked(vals[3], vals[4], vals[5]) {
                Some((r, g, b)) => Parsed::ok(
                    CMD_RANGE,
                    LedCommand::Range {
                        start: vals[1],
                        end: vals[2],
                        r,
                        g,
                        b,
                    },
                ),
                None => Parsed::err(CMD_RANGE, D_RANGE, true),
            }
        }
        4 => {
            if count < 4 {
                return Parsed::err(CMD_COLOR, D_BAD_ARGS, true);
            }
            match rgb_checked(vals[1], vals[2], vals[3]) {
                Some((r, g, b)) => Parsed::ok(CMD_COLOR, LedCommand::Color { r, g, b }),
                None => Parsed::err(CMD_COLOR, D_RANGE, true),
            }
        }
        5 => {
            if count < 2 {
                return Parsed::err(CMD_BRIGHTNESS, D_BAD_ARGS, true);
            }
            match u8::try_from(vals[1]) {
                Ok(value) => Parsed::ok(CMD_BRIGHTNESS, LedCommand::Brightness { value }),
                Err(_) => Parsed::err(CMD_BRIGHTNESS, D_RANGE, true),
            }
        }
        99 => Parsed::ok(CMD_NOOP, LedCommand::Noop),
        _ => Parsed::err(cmd as u8, D_UNKNOWN, true),
    }
}

fn rgb_checked(r: i32, g: i32, b: i32) -> Option<(u8, u8, u8)> {
    let r = u8::try_from(r).ok()?;
    let g = u8::try_from(g).ok()?;
    let b = u8::try_from(b).ok()?;
    Some((r, g, b))
}

/// Typed constructors used by the HTTP controllers. They run the same
/// validation as parse_ascii, so e.g. `color(-5, 0, 0)` produces the same
/// error ACK the Arduino answered with.
pub fn color(r: i32, g: i32, b: i32) -> Parsed {
    match rgb_checked(r, g, b) {
        Some((r, g, b)) => Parsed::ok(CMD_COLOR, LedCommand::Color { r, g, b }),
        None => Parsed::err(CMD_COLOR, D_RANGE, true),
    }
}

pub fn brightness(value: i32) -> Parsed {
    match u8::try_from(value) {
        Ok(value) => Parsed::ok(CMD_BRIGHTNESS, LedCommand::Brightness { value }),
        Err(_) => Parsed::err(CMD_BRIGHTNESS, D_RANGE, true),
    }
}

pub fn range(start: i32, end: i32, r: i32, g: i32, b: i32) -> Parsed {
    match rgb_checked(r, g, b) {
        Some((r, g, b)) => Parsed::ok(
            CMD_RANGE,
            LedCommand::Range {
                start,
                end,
                r,
                g,
                b,
            },
        ),
        None => Parsed::err(CMD_RANGE, D_RANGE, true),
    }
}

pub fn rainbow() -> Parsed {
    Parsed::ok(CMD_RAINBOW, LedCommand::Rainbow)
}

pub fn off() -> Parsed {
    Parsed::ok(CMD_OFF, LedCommand::Off)
}

#[derive(Serialize)]
struct AckJson<'a> {
    ts: u64,
    status: &'a str,
    command: &'a str,
    detail: &'a str,
    status_code: u8,
    command_code: u8,
    detail_code: u8,
}

fn status_name(b: u8) -> &'static str {
    match b {
        ST_OK => "ok",
        ST_ERR => "err",
        _ => "unknown",
    }
}

fn detail_name(b: u8) -> &'static str {
    match b {
        D_NONE => "none",
        D_UNKNOWN => "unknown",
        D_BAD_ARGS => "bad_args",
        D_RANGE => "range",
        D_INTERNAL => "internal",
        _ => "unknown",
    }
}

fn cmd_name(b: u8) -> &'static str {
    match b {
        CMD_OFF_RAW => "off",
        CMD_RAINBOW => "rainbow",
        0x02 => "-reserved-",
        CMD_RANGE => "set range",
        CMD_COLOR => "set rgb",
        CMD_BRIGHTNESS => "brightness",
        CMD_OFF => "switch off",
        0x12 => "-reserved-",
        _ => "unknown",
    }
}

/// Renders the ACK as the JSON broadcast over SSE — same shape the Arduino's
/// 3-byte notifications used to produce.
pub fn ack_json(parsed: &Parsed) -> String {
    let obj = AckJson {
        ts: chrono::Utc::now().timestamp_millis() as u64,
        status: status_name(parsed.status),
        command: cmd_name(parsed.cmd_code),
        detail: detail_name(parsed.detail),
        status_code: parsed.status,
        command_code: parsed.cmd_code,
        detail_code: parsed.detail,
    };
    serde_json::to_string(&obj).unwrap_or_else(|_| "err.malformed-json".to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        CMD_BRIGHTNESS, CMD_COLOR, CMD_NOOP, CMD_OFF_RAW, CMD_RAINBOW, CMD_RANGE, CMD_UNPARSED,
        D_BAD_ARGS, D_NONE, D_RANGE, D_UNKNOWN, LedCommand, Parsed, ST_ERR, ST_OK, parse_ascii,
    };

    fn ok(parsed: &Parsed) -> &LedCommand {
        assert_eq!(parsed.status, ST_OK);
        assert_eq!(parsed.detail, D_NONE);
        assert!(parsed.stops_rainbow);
        parsed.command.as_ref().unwrap()
    }

    fn err(parsed: &Parsed, cmd: u8, detail: u8, stops: bool) {
        assert_eq!(parsed.status, ST_ERR);
        assert_eq!(parsed.cmd_code, cmd);
        assert_eq!(parsed.detail, detail);
        assert_eq!(parsed.stops_rainbow, stops);
        assert!(parsed.command.is_none());
    }

    #[test]
    fn parses_every_documented_command() {
        assert_eq!(*ok(&parse_ascii("0")), LedCommand::Off);
        assert_eq!(parse_ascii("0").cmd_code, CMD_OFF_RAW);
        assert_eq!(*ok(&parse_ascii("1")), LedCommand::Rainbow);
        assert_eq!(parse_ascii("1").cmd_code, CMD_RAINBOW);
        assert_eq!(*ok(&parse_ascii("99")), LedCommand::Noop);
        assert_eq!(parse_ascii("99").cmd_code, CMD_NOOP);

        assert_eq!(
            *ok(&parse_ascii("4 255 0 0")),
            LedCommand::Color { r: 255, g: 0, b: 0 }
        );
        assert_eq!(
            *ok(&parse_ascii("3 10 20 128 128 128")),
            LedCommand::Range {
                start: 10,
                end: 20,
                r: 128,
                g: 128,
                b: 128
            }
        );
        assert_eq!(
            *ok(&parse_ascii("5 30")),
            LedCommand::Brightness { value: 30 }
        );
        assert_eq!(parse_ascii("5 30").cmd_code, CMD_BRIGHTNESS);
        assert_eq!(parse_ascii("4 1 2 3").cmd_code, CMD_COLOR);
        assert_eq!(parse_ascii("3 1 2 3 4 5").cmd_code, CMD_RANGE);
    }

    #[test]
    fn trailing_newline_and_extra_args_are_ignored() {
        assert_eq!(
            *ok(&parse_ascii("4 1 2 3\n")),
            LedCommand::Color { r: 1, g: 2, b: 3 }
        );
        assert_eq!(
            *ok(&parse_ascii("4 1 2 3 9 9")),
            LedCommand::Color { r: 1, g: 2, b: 3 }
        );
    }

    #[test]
    fn rgb_and_brightness_validation() {
        err(&parse_ascii("4 256 0 0"), CMD_COLOR, D_RANGE, true);
        err(&parse_ascii("4 -1 0 0"), CMD_COLOR, D_RANGE, true);
        err(&parse_ascii("4 1 2"), CMD_COLOR, D_BAD_ARGS, true);
        err(&parse_ascii("3 0 0 300 0 0"), CMD_RANGE, D_RANGE, true);
        err(&parse_ascii("3 0 0 0 0"), CMD_RANGE, D_BAD_ARGS, true);
        err(&parse_ascii("5 300"), CMD_BRIGHTNESS, D_RANGE, true);
        err(&parse_ascii("5 -1"), CMD_BRIGHTNESS, D_RANGE, true);
        err(&parse_ascii("5"), CMD_BRIGHTNESS, D_BAD_ARGS, true);
        // Brightness 0 is valid: strip goes dark, framebuffer keeps colors.
        assert_eq!(
            *ok(&parse_ascii("5 0")),
            LedCommand::Brightness { value: 0 }
        );
    }

    #[test]
    fn range_indices_are_not_validated() {
        // Reversed and out-of-bounds indices parse fine; the engine clamps.
        let parsed = parse_ascii("3 300 -10 1 2 3");
        match *ok(&parsed) {
            LedCommand::Range { start, end, .. } => {
                assert_eq!(start, 300);
                assert_eq!(end, -10);
            }
            _ => panic!("expected range"),
        }
    }

    #[test]
    fn unknown_and_malformed_commands() {
        err(&parse_ascii("2"), 2, D_UNKNOWN, true);
        err(&parse_ascii("7"), 7, D_UNKNOWN, true);
        err(&parse_ascii(""), CMD_UNPARSED, D_BAD_ARGS, false);
        err(&parse_ascii("   "), CMD_UNPARSED, D_BAD_ARGS, false);
        err(&parse_ascii("abc"), CMD_UNPARSED, D_BAD_ARGS, false);
        // Ids out of 0..=99 echo 0xFF and do NOT stop the rainbow.
        err(&parse_ascii("100"), CMD_UNPARSED, D_RANGE, false);
        err(&parse_ascii("-1"), CMD_UNPARSED, D_RANGE, false);
        // Garbage after a valid id still stops the rainbow (bad args).
        err(&parse_ascii("4 x y z"), CMD_COLOR, D_BAD_ARGS, true);
    }

    #[test]
    fn typed_constructors_match_ascii_validation() {
        assert_eq!(
            super::color(10, 20, 30).command,
            parse_ascii("4 10 20 30").command
        );
        err(&super::color(256, 0, 0), CMD_COLOR, D_RANGE, true);
        err(&super::brightness(999), CMD_BRIGHTNESS, D_RANGE, true);
        assert_eq!(
            super::range(1, 2, 3, 4, 5).command,
            parse_ascii("3 1 2 3 4 5").command
        );
        assert_eq!(*ok(&super::off()), LedCommand::Off);
        assert_eq!(super::off().cmd_code, super::CMD_OFF);
    }

    #[test]
    fn ack_json_keeps_the_documented_shape() {
        let json = super::ack_json(&parse_ascii("4 1 2 3"));
        let obj: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(obj["status"], "ok");
        assert_eq!(obj["command"], "set rgb");
        assert_eq!(obj["detail"], "none");
        assert_eq!(obj["status_code"], 0);
        assert_eq!(obj["command_code"], 4);
        assert_eq!(obj["detail_code"], 0);
        assert!(obj["ts"].as_u64().unwrap() > 0);
    }
}
