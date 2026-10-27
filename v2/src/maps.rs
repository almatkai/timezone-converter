use base64::Engine;
pub fn image(offset: i32) -> Option<String> {
    let bytes: &[u8] = match offset {
        3600 => include_bytes!("../resources/timezones/UTC+1.png"),
        36000 => include_bytes!("../resources/timezones/UTC+10.png"),
        39600 => include_bytes!("../resources/timezones/UTC+11.png"),
        43200 => include_bytes!("../resources/timezones/UTC+12.png"),
        7200 => include_bytes!("../resources/timezones/UTC+2.png"),
        10800 => include_bytes!("../resources/timezones/UTC+3.png"),
        14400 => include_bytes!("../resources/timezones/UTC+4.png"),
        18000 => include_bytes!("../resources/timezones/UTC+5.png"),
        21600 => include_bytes!("../resources/timezones/UTC+6.png"),
        25200 => include_bytes!("../resources/timezones/UTC+7.png"),
        28800 => include_bytes!("../resources/timezones/UTC+8.png"),
        32400 => include_bytes!("../resources/timezones/UTC+9.png"),
        -3600 => include_bytes!("../resources/timezones/UTC-1.png"),
        -36000 => include_bytes!("../resources/timezones/UTC-10.png"),
        -39600 => include_bytes!("../resources/timezones/UTC-11.png"),
        -43200 => include_bytes!("../resources/timezones/UTC-12.png"),
        -7200 => include_bytes!("../resources/timezones/UTC-2.png"),
        -10800 => include_bytes!("../resources/timezones/UTC-3.png"),
        -14400 => include_bytes!("../resources/timezones/UTC-4.png"),
        -18000 => include_bytes!("../resources/timezones/UTC-5.png"),
        -21600 => include_bytes!("../resources/timezones/UTC-6.png"),
        -25200 => include_bytes!("../resources/timezones/UTC-7.png"),
        -28800 => include_bytes!("../resources/timezones/UTC-8.png"),
        -32400 => include_bytes!("../resources/timezones/UTC-9.png"),
        0 => include_bytes!("../resources/timezones/UTC.png"),
        _ => return None,
    };
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}
