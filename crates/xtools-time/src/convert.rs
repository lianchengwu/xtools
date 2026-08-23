use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;

/// Local display: space, no T, no Z, millisecond fraction.
pub const LOCAL_FMT: &str = "%Y-%m-%d %H:%M:%S%.3f";
pub const SECOND_FMT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimezoneOption {
    pub label: &'static str,
    pub iana_name: Option<&'static str>,
    pub fallback_offset_hours: i8,
}

pub const TIMEZONE_OPTIONS: &[TimezoneOption] = &[
    TimezoneOption {
        label: "本地时间 (Local)",
        iana_name: None,
        fallback_offset_hours: 0,
    },
    TimezoneOption {
        label: "UTC (世界标准时间)",
        iana_name: Some("UTC"),
        fallback_offset_hours: 0,
    },
    TimezoneOption {
        label: "UTC+8 (北京 / 上海 / 香港 / 台北)",
        iana_name: Some("Asia/Shanghai"),
        fallback_offset_hours: 8,
    },
    TimezoneOption {
        label: "UTC+9 (东京 / 首尔)",
        iana_name: Some("Asia/Tokyo"),
        fallback_offset_hours: 9,
    },
    TimezoneOption {
        label: "UTC+7 (曼谷 / 雅加达 / 河内)",
        iana_name: Some("Asia/Bangkok"),
        fallback_offset_hours: 7,
    },
    TimezoneOption {
        label: "UTC+0 (伦敦 / GMT / BST)",
        iana_name: Some("Europe/London"),
        fallback_offset_hours: 0,
    },
    TimezoneOption {
        label: "UTC+1 (巴黎 / 柏林 / 罗马)",
        iana_name: Some("Europe/Berlin"),
        fallback_offset_hours: 1,
    },
    TimezoneOption {
        label: "UTC+3 (莫斯科 / 利雅得)",
        iana_name: Some("Europe/Moscow"),
        fallback_offset_hours: 3,
    },
    TimezoneOption {
        label: "UTC+10 (悉尼 / 墨尔本)",
        iana_name: Some("Australia/Sydney"),
        fallback_offset_hours: 10,
    },
    TimezoneOption {
        label: "UTC-5 (纽约 / 美东 EST/EDT)",
        iana_name: Some("America/New_York"),
        fallback_offset_hours: -5,
    },
    TimezoneOption {
        label: "UTC-6 (芝加哥 / 中部 CST/CDT)",
        iana_name: Some("America/Chicago"),
        fallback_offset_hours: -6,
    },
    TimezoneOption {
        label: "UTC-7 (丹佛 / 山区 MST/MDT)",
        iana_name: Some("America/Denver"),
        fallback_offset_hours: -7,
    },
    TimezoneOption {
        label: "UTC-8 (旧金山 / 美西 PST/PDT)",
        iana_name: Some("America/Los_Angeles"),
        fallback_offset_hours: -8,
    },
];

pub fn resolve_timezone_by_index(index: usize) -> TimeZone {
    let opt = TIMEZONE_OPTIONS.get(index).unwrap_or(&TIMEZONE_OPTIONS[0]);
    resolve_timezone(opt)
}

pub fn resolve_timezone(opt: &TimezoneOption) -> TimeZone {
    match opt.iana_name {
        None => TimeZone::system(),
        Some(name) => TimeZone::get(name).unwrap_or_else(|_| {
            let offset_sec = (opt.fallback_offset_hours as i32) * 3600;
            let offset = jiff::tz::Offset::from_seconds(offset_sec)
                .unwrap_or(jiff::tz::Offset::UTC);
            TimeZone::fixed(offset)
        }),
    }
}

pub fn from_now(tz: &TimeZone) -> (i64, i64, String) {
    format_ts(Timestamp::now(), tz)
}

pub fn from_seconds(s: i64, tz: &TimeZone) -> Result<(i64, i64, String), jiff::Error> {
    Ok(format_ts(Timestamp::from_second(s)?, tz))
}

pub fn from_millis(ms: i64, tz: &TimeZone) -> Result<(i64, i64, String), jiff::Error> {
    Ok(format_ts(Timestamp::from_millisecond(ms)?, tz))
}

pub fn from_datetime(text: &str, tz: &TimeZone) -> Result<(i64, i64, String), jiff::Error> {
    let trimmed = text.trim();

    // 1. Try parsing full ISO8601/RFC3339 timestamp with timezone (e.g. 2026-08-24T15:30:00Z or +08:00)
    if let Ok(ts) = trimmed.parse::<Timestamp>() {
        return Ok(format_ts(ts, tz));
    }
    if let Ok(zoned) = trimmed.parse::<jiff::Zoned>() {
        return Ok(format_ts(zoned.timestamp(), tz));
    }

    // 2. Try parsing civil datetime
    let dt = DateTime::strptime(LOCAL_FMT, trimmed)
        .or_else(|_| DateTime::strptime(SECOND_FMT, trimmed))
        .or_else(|_| trimmed.parse::<DateTime>())?;

    let zoned = tz.to_zoned(dt)?;
    Ok(format_ts(zoned.timestamp(), tz))
}

pub fn format_ts(ts: Timestamp, tz: &TimeZone) -> (i64, i64, String) {
    let zoned = ts.to_zoned(tz.clone());
    let formatted = zoned.strftime(LOCAL_FMT).to_string();
    (ts.as_second(), ts.as_millisecond(), formatted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_conversion_roundtrip() {
        let tz = TimeZone::UTC;
        let (s, ms, utc) = from_seconds(1700000000, &tz).unwrap();
        assert_eq!(s, 1700000000);
        assert_eq!(ms, 1700000000000);
        assert_eq!(utc, "2023-11-14 22:13:20.000");
    }

    #[test]
    fn millis_conversion_roundtrip() {
        let tz = TimeZone::UTC;
        let (s, ms, utc) = from_millis(1700000000123, &tz).unwrap();
        assert_eq!(s, 1700000000);
        assert_eq!(ms, 1700000000123);
        assert_eq!(utc, "2023-11-14 22:13:20.123");
    }

    #[test]
    fn datetime_conversion_roundtrip() {
        let tz = resolve_timezone_by_index(2); // UTC+8
        let (_, _, dt_str) = from_seconds(1700000000, &tz).unwrap();
        assert_eq!(dt_str, "2023-11-15 06:13:20.000");

        let (s2, _, _) = from_datetime(&dt_str, &tz).unwrap();
        assert_eq!(s2, 1700000000);
    }

    #[test]
    fn timezone_conversion_differences() {
        let utc_tz = TimeZone::UTC;
        let shanghai_tz = TimeZone::get("Asia/Shanghai").unwrap();
        let tokyo_tz = TimeZone::get("Asia/Tokyo").unwrap();
        let ny_tz = TimeZone::get("America/New_York").unwrap();

        let (_, _, utc_str) = from_seconds(1700000000, &utc_tz).unwrap();
        let (_, _, sh_str) = from_seconds(1700000000, &shanghai_tz).unwrap();
        let (_, _, tk_str) = from_seconds(1700000000, &tokyo_tz).unwrap();
        let (_, _, ny_str) = from_seconds(1700000000, &ny_tz).unwrap();

        assert_eq!(utc_str, "2023-11-14 22:13:20.000");
        assert_eq!(sh_str, "2023-11-15 06:13:20.000"); // UTC+8 (+8h)
        assert_eq!(tk_str, "2023-11-15 07:13:20.000"); // UTC+9 (+9h)
        assert_eq!(ny_str, "2023-11-14 17:13:20.000"); // UTC-5 in Nov (EST: -5h)
    }

    #[test]
    fn rfc3339_input_parsed() {
        let tz = TimeZone::UTC;
        let (s, _, _) = from_datetime("2023-11-14T22:13:20Z", &tz).unwrap();
        assert_eq!(s, 1700000000);

        let (s2, _, _) = from_datetime("2023-11-15T06:13:20+08:00", &tz).unwrap();
        assert_eq!(s2, 1700000000);
    }

    #[test]
    fn invalid_datetime_rejected() {
        let tz = TimeZone::UTC;
        assert!(from_datetime("invalid-date-string", &tz).is_err());
    }
}
