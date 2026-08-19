//! Dependency-free UTC ISO-8601 timestamp formatting, so this crate keeps
//! the empty-`[dependencies]` convention already used by `my-lisp`/`cml`'s
//! crates rather than pulling in `chrono`/`time` for a single formatter.

/// Current UTC time as `YYYY-MM-DDTHH:MM:SS.mmmZ`.
pub fn iso8601_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format_iso8601(dur.as_secs() as i64, dur.subsec_millis())
}

fn format_iso8601(unix_secs: i64, millis: u32) -> String {
    let days = unix_secs.div_euclid(86400);
    let secs_of_day = unix_secs.rem_euclid(86400);
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

/// Howard Hinnant's `civil_from_days` algorithm (public domain), ported
/// from the reference C++ implementation. Converts a day count since the
/// Unix epoch (1970-01-01) into a proleptic-Gregorian (year, month, day)
/// triple. Correct for negative `z` (pre-1970 dates) as well.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference epoch-day counts verified independently via
    // `date -u -d '<date>' +%s` / 86400 before writing this table — not
    // taken from memory of the Hinnant algorithm's own test suite.
    #[test]
    fn civil_from_days_matches_verified_references() {
        let cases: &[(i64, (i64, u32, u32))] = &[
            (0, (1970, 1, 1)),
            (11016, (2000, 2, 29)), // leap day, century-divisible-by-400 year
            (19782, (2024, 2, 29)), // leap day, ordinary leap year
            (20684, (2026, 8, 19)), // "today" per this session's context
            (-1, (1969, 12, 31)),   // pre-epoch, exercises the negative-z branch
            (47541, (2100, 3, 1)),  // 2100 is NOT a leap year (divisible by 100, not 400)
        ];
        for &(days, expected) in cases {
            assert_eq!(civil_from_days(days), expected, "days={days}");
        }
    }

    #[test]
    fn format_iso8601_shape() {
        let s = format_iso8601(20684 * 86400 + 12 * 3600 + 34 * 60 + 56, 789);
        assert_eq!(s, "2026-08-19T12:34:56.789Z");
    }
}
