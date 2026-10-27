//! Arithmetic is independent of UI, storage and the system clock.
use chrono::{DateTime, Datelike, Duration, NaiveDateTime, Timelike, Utc};
type Tz = tz::TimeZoneRef<'static>;
use serde::Serialize;
use std::{collections::BTreeMap, sync::OnceLock};
pub const TZDB: &str = tzdb::VERSION;
pub fn catalog() -> &'static [String] {
    static Z: OnceLock<Vec<String>> = OnceLock::new();
    Z.get_or_init(|| {
        serde_json::from_str(include_str!("../resources/catalog.json")).expect("bundled catalog")
    })
}
pub fn zone(s: &str) -> Result<Tz, String> {
    static LINKS: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    let links = LINKS.get_or_init(|| {
        serde_json::from_str(include_str!("../resources/iana-links.json"))
            .expect("bundled IANA links")
    });
    let mut canonical = s;
    // IANA backward links match ICU's non-backzone historical model.
    for _ in 0..8 {
        match links.get(canonical) {
            Some(v) => canonical = v,
            None => {
                return tzdb::tz_by_name(canonical)
                    .ok_or_else(|| format!("Timezone is not in the pinned IANA database: {s}"))
            }
        }
    }
    Err("Pinned IANA link chain exceeds 8 entries".into())
}
pub fn instant(ms: i64) -> Result<DateTime<Utc>, String> {
    DateTime::from_timestamp_millis(ms).ok_or_else(|| "Instant is outside chrono's representable date range (approximately -262143 to +262142 years)".into())
}
pub fn name(s: &str) -> String {
    s.replace('/', " - ").replace('_', " ")
}
pub fn offset(ms: i64, z: &str) -> Result<i32, String> {
    instant(ms)?;
    zone(z)?
        .find_local_time_type(ms.div_euclid(1000))
        .map(|t| t.ut_offset())
        .map_err(|e| format!("IANA timezone conversion failed: {e}"))
}
pub fn gmt(ms: i64, z: &str) -> Result<String, String> {
    let n = offset(ms, z)? as f64 / 3600.;
    Ok(format!("GMT{}{n}", if n < 0. { "" } else { "+" }))
}
fn intl_gmt(secs: i32) -> String {
    if secs == 0 {
        return "GMT".into();
    }
    let a = secs.unsigned_abs();
    let mut s = format!("GMT{}{}", if secs < 0 { '-' } else { '+' }, a / 3600);
    if a % 3600 != 0 {
        s.push_str(&format!(":{:02}", a / 60 % 60));
    }
    if a % 60 != 0 {
        s.push_str(&format!(":{:02}", a % 60));
    }
    s
}
pub fn short(ms: i64, z: &str) -> Result<String, String> {
    type Interval = (Option<i64>, Option<i64>, String);
    #[derive(serde::Deserialize)]
    struct Cldr {
        intervals: BTreeMap<String, Vec<Interval>>,
        names: BTreeMap<String, BTreeMap<String, String>>,
    }
    static DATA: OnceLock<Cldr> = OnceLock::new();
    let t = zone(z)?
        .find_local_time_type(ms.div_euclid(1000))
        .map_err(|e| e.to_string())?;
    let secs = t.ut_offset();
    if z == "UTC" || z == "Etc/UTC" {
        return Ok("UTC".into());
    }
    let data = DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../resources/cldr-zones.json"))
            .expect("bundled CLDR names")
    });
    if ms >= 0 {
        if let Some(intervals) = data.intervals.get(z) {
            if let Some((_, _, meta)) = intervals
                .iter()
                .find(|(from, to, _)| from.is_none_or(|n| ms >= n) && to.is_none_or(|n| ms < n))
            {
                if meta == "GMT" && secs == 0 {
                    return Ok("GMT".into());
                }
                if meta != "GMT" || secs == 0 {
                    if let Some(s) = data
                        .names
                        .get(meta)
                        .and_then(|n| n.get(if t.is_dst() { "daylight" } else { "standard" }))
                    {
                        return Ok(s.clone());
                    }
                }
            }
        }
    }
    Ok(if secs == 0 {
        "GMT+0".into()
    } else {
        intl_gmt(secs)
    })
}
#[derive(Debug, Serialize)]
pub struct Row {
    pub zone: String,
    pub ff: String,
    pub short: String,
    #[serde(rename = "offsetSeconds")]
    pub offset_seconds: i32,
}
pub fn row(ms: i64, z: &str) -> Result<Row, String> {
    let secs = offset(ms, z)?;
    let d = local_wall(ms, zone(z)?)?;
    let year = d.year();
    // Intl Gregorian formatting represents BCE as the positive year-of-era.
    let y = if year <= 0 { 1 - year } else { year };
    let h = d.hour();
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    Ok(Row {
        zone: z.into(),
        ff: format!(
            "{} {}, {}, {}:{:02} {}",
            months[d.month0() as usize],
            d.day(),
            y,
            if h % 12 == 0 { 12 } else { h % 12 },
            d.minute(),
            if h < 12 { "AM" } else { "PM" }
        ),
        short: short(ms, z)?,
        offset_seconds: secs,
    })
}
pub fn wall(ms: i64, z: &str) -> Result<String, String> {
    Ok(local_wall(ms, zone(z)?)?
        .format("%Y-%m-%dT%H:%M:%S%.3f")
        .to_string())
}
pub fn parse_wall(s: &str) -> Result<NaiveDateTime, String> {
    if s.len() > 40 {
        return Err("Date/time exceeds 40 UTF-8 bytes".into());
    }
    ["%Y-%m-%dT%H:%M:%S%.f","%Y-%m-%dT%H:%M","%Y-%m-%d %H:%M:%S%.f","%Y-%m-%d %H:%M"].iter().find_map(|f|NaiveDateTime::parse_from_str(s,f).ok()).filter(|d|d.nanosecond()<1_000_000_000&&d.nanosecond()%1_000_000==0).ok_or_else(||"Enter local date/time as YYYY-MM-DDTHH:mm[:ss[.SSS]] (millisecond precision, no leap seconds)".into())
}
/// ECMAScript DatePicker equivalent: earlier fold, forward through a gap.
pub fn picker(s: &str, local: &str) -> Result<i64, String> {
    if s.len() > 40 {
        return Err("Date/time exceeds 40 UTF-8 bytes".into());
    }
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        if d.nanosecond() >= 1_000_000_000 || d.nanosecond() % 1_000_000 != 0 {
            return Err("Use millisecond precision without leap seconds".into());
        }
        let ms = d.timestamp_millis();
        instant(ms)?;
        return Ok(ms);
    }
    let wall = parse_wall(s)?;
    let tz = zone(local)?;
    // Resolve with the pre-transition offset; on folds choose the earlier instant.
    let guess = guess_near(wall, tz)?;
    let resolved = resolve(wall, tz, guess)?;
    let mut candidates = vec![resolved];
    for hours in [-48, 48] {
        if let Some(near) = wall.checked_add_signed(Duration::hours(hours)) {
            let g = offset_tz(near.and_utc().timestamp_millis(), tz)?;
            let u = resolve(wall, tz, g)?;
            if local_wall(u, tz)? == wall {
                candidates.push(u);
            }
        }
    }
    candidates.sort();
    Ok(candidates[0])
}
fn guess_near(wall: NaiveDateTime, z: Tz) -> Result<i32, String> {
    // A 48-hour look-behind covers IANA's largest civil gap (24 hours).
    let near = wall
        .checked_sub_signed(Duration::hours(48))
        .ok_or("Date is too close to lower representable boundary")?;
    offset_tz(near.and_utc().timestamp_millis(), z)
}
/// Luxon 3.3.0 fixOffset: preserve offset guess in folds, advance gaps.
fn resolve(wall: NaiveDateTime, z: Tz, guess: i32) -> Result<i64, String> {
    let local = wall.and_utc().timestamp_millis();
    let mut utc = local
        .checked_sub(i64::from(guess) * 1000)
        .ok_or("Date arithmetic overflow")?;
    let o2 = offset_tz(utc, z)?;
    if o2 == guess {
        return Ok(utc);
    }
    utc = utc
        .checked_sub(i64::from(o2 - guess) * 1000)
        .ok_or("Date arithmetic overflow")?;
    let o3 = offset_tz(utc, z)?;
    if o2 == o3 {
        return Ok(utc);
    }
    let utc = local
        .checked_sub(i64::from(o2.min(o3)) * 1000)
        .ok_or("Date arithmetic overflow")?;
    instant(utc)?;
    Ok(utc)
}
pub fn custom(picker_ms: i64, selected: &str, local: &str) -> Result<i64, String> {
    let local_wall = local_wall(picker_ms, zone(local)?)?;
    resolve(local_wall, zone(selected)?, offset(picker_ms, selected)?)
}

fn offset_tz(ms: i64, z: Tz) -> Result<i32, String> {
    instant(ms)?;
    z.find_local_time_type(ms.div_euclid(1000))
        .map(|t| t.ut_offset())
        .map_err(|e| e.to_string())
}
fn local_wall(ms: i64, z: Tz) -> Result<NaiveDateTime, String> {
    instant(ms)?
        .naive_utc()
        .checked_add_signed(Duration::seconds(i64::from(offset_tz(ms, z)?)))
        .ok_or_else(|| "Local date is outside the representable boundary".into())
}
