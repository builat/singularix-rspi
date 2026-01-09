use serde::Serialize;

#[derive(Serialize)]
struct BleAckJson<'a> {
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
        0x00 => "ok",
        0x01 => "err",
        _ => "unknown",
    }
}

fn detail_name(b: u8) -> &'static str {
    match b {
        0x00 => "none",
        0x01 => "unknown",
        0x02 => "bad_args",
        0x03 => "range",
        0x04 => "internal",
        _ => "unknown",
    }
}

fn cmd_name(b: u8) -> &'static str {
    match b {
        0x01 => "rainbow",
        0x02 => "-reserved-",
        0x03 => "set range",
        0x10 => "set rgb",
        0x11 => "switch off",
        0x12 => "-reserved-",
        _ => "unknown",
    }
}

pub fn parse_arduino_resp(payload: &[u8]) -> Option<String> {
    if payload.len() != 3 {
        return None;
    }
    let status = payload[0];
    let cmd = payload[1];
    let detail = payload[2];

    let obj = BleAckJson {
        ts: chrono::Utc::now().timestamp_millis() as u64,
        status: status_name(status),
        command: cmd_name(cmd),
        detail: detail_name(detail),
        status_code: status,
        command_code: cmd,
        detail_code: detail,
    };
    serde_json::to_string(&obj).ok()
}
