use jiff::Timestamp;
use jiff::civil::DateTime;
use jiff::tz::TimeZone;

/// Local display: space, no T, no Z, millisecond fraction.
pub const LOCAL_FMT: &str = "%Y-%m-%d %H:%M:%S%.3f";

pub fn from_now() -> (i64, i64, String) {
    format_ts(Timestamp::now())
}

pub fn from_seconds(s: i64) -> Result<(i64, i64, String), jiff::Error> {
    Ok(format_ts(Timestamp::from_second(s)?))
}

pub fn from_millis(ms: i64) -> Result<(i64, i64, String), jiff::Error> {
    Ok(format_ts(Timestamp::from_millisecond(ms)?))
}

pub fn from_local(text: &str) -> Result<(i64, i64, String), jiff::Error> {
    let trimmed = text.trim();
    let dt = DateTime::strptime(LOCAL_FMT, trimmed).or_else(|_| trimmed.parse::<DateTime>())?;
    let zoned = TimeZone::system().to_zoned(dt)?;
    Ok(format_ts(zoned.timestamp()))
}

fn format_ts(ts: Timestamp) -> (i64, i64, String) {
    let zoned = ts.to_zoned(TimeZone::system());
    let local = zoned.strftime(LOCAL_FMT).to_string();
    (ts.as_second(), ts.as_millisecond(), local)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seconds_conversion_roundtrip() {
        let (s, ms, local) = from_seconds(1700000000).unwrap();
        assert_eq!(s, 1700000000);
        assert_eq!(ms, 1700000000000);
        assert!(!local.is_empty());
    }

    #[test]
    fn millis_conversion_roundtrip() {
        let (s, ms, local) = from_millis(1700000000123).unwrap();
        assert_eq!(s, 1700000000);
        assert_eq!(ms, 1700000000123);
        assert!(!local.is_empty());
    }

    #[test]
    fn local_conversion_roundtrip() {
        let (_, _, local) = from_seconds(1700000000).unwrap();
        let (s2, _, _) = from_local(&local).unwrap();
        assert_eq!(s2, 1700000000);
    }

    #[test]
    fn invalid_local_rejected() {
        assert!(from_local("invalid-date-string").is_err());
    }
}
