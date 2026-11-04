//! Calendar dates in the contract, always written as `YYYY-MM-DD`.

use serde::{Serialize, Serializer};
use time::{Date, Month};

/// A real calendar date with a four-digit year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PublishedDate(Date);

impl PublishedDate {
    /// Reads exactly `YYYY-MM-DD`; anything else, or a date that does not
    /// exist (2026-02-30), is `None`.
    pub fn parse(raw: &str) -> Option<Self> {
        // ASCII first, so the byte slicing below cannot split a character.
        let bytes = raw.as_bytes();
        if !raw.is_ascii() || bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return None;
        }
        let year: i32 = digits(&raw[0..4])?;
        let month: u8 = digits(&raw[5..7])?;
        let day: u8 = digits(&raw[8..10])?;
        let month = Month::try_from(month).ok()?;
        Date::from_calendar_date(year, month, day).ok().map(Self)
    }

    pub fn from_date(date: Date) -> Option<Self> {
        (1000..=9999).contains(&date.year()).then_some(Self(date))
    }

    pub fn date(self) -> Date {
        self.0
    }
}

impl std::fmt::Display for PublishedDate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.0.year(),
            u8::from(self.0.month()),
            self.0.day()
        )
    }
}

impl Serialize for PublishedDate {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

/// Only ASCII digits: `str::parse` would also take a leading `+`.
fn digits<T: std::str::FromStr>(raw: &str) -> Option<T> {
    if raw.bytes().all(|byte| byte.is_ascii_digit()) {
        raw.parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_date_round_trips() {
        let date = PublishedDate::parse("2026-11-03").unwrap();
        assert_eq!(date.to_string(), "2026-11-03");
        assert_eq!(serde_json::to_string(&date).unwrap(), r#""2026-11-03""#);
    }

    #[test]
    fn malformed_or_impossible_dates_are_none() {
        for raw in [
            "",
            "2026-13-01",
            "2026-02-30",
            "2026-1-01",
            "26-11-03",
            "2026/11/03",
            "+026-11-03",
            "2026-11-03T00:00:00Z",
            "２０２６-11-03",
            "20é6-11-03",
        ] {
            assert_eq!(PublishedDate::parse(raw), None, "input: {raw}");
        }
    }

    #[test]
    fn leap_day_exists_only_in_leap_years() {
        assert!(PublishedDate::parse("2028-02-29").is_some());
        assert!(PublishedDate::parse("2026-02-29").is_none());
    }

    #[test]
    fn dates_outside_four_digit_years_are_refused() {
        let date = Date::from_calendar_date(999, Month::January, 1).unwrap();
        assert_eq!(PublishedDate::from_date(date), None);
        let date = Date::from_calendar_date(2026, Month::January, 1).unwrap();
        assert_eq!(PublishedDate::from_date(date).unwrap().date(), date);
    }
}
