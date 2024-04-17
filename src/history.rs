use crate::Result;
use chrono::NaiveDate;

#[derive(Debug, Clone, Copy)]
pub struct HistoryRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}
impl HistoryRange {
    pub fn new(start: &str, end: &str, today: NaiveDate) -> Result<Self> {
        fn parse(value: &str) -> Result<NaiveDate> {
            let date = NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .map_err(|_| "dates must be valid YYYY-MM-DD values".to_string())?;
            if date.to_string() != value {
                return Err("dates must use zero-padded YYYY-MM-DD".into());
            }
            Ok(date)
        }
        let start = parse(start)?;
        let end = parse(end)?;
        if start > end {
            return Err("history start must not be after end".into());
        }
        if end > today {
            return Err("historical dates cannot be in the future".into());
        }
        if start < NaiveDate::from_ymd_opt(1940, 1, 1).ok_or("invalid archive epoch")? {
            return Err("archive data begins in 1940".into());
        }
        if (end - start).num_days() >= 366 {
            return Err("history requests are limited to 366 days".into());
        }
        Ok(Self { start, end })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
    }
    #[test]
    fn accepts_leap_day_and_a_single_day() {
        let range = HistoryRange::new("2024-02-29", "2024-02-29", today()).unwrap();
        assert_eq!(range.start.to_string(), "2024-02-29");
    }
    #[test]
    fn rejects_invalid_reversed_future_and_oversized_ranges() {
        for (start, end) in [
            ("2023-02-29", "2023-03-01"),
            ("2024-03-01", "2024-02-29"),
            ("2024-03-01", "2024-03-02"),
            ("2020-01-01", "2024-01-01"),
            ("1900-01-01", "1900-01-02"),
            ("2024-2-01", "2024-02-02"),
        ] {
            assert!(
                HistoryRange::new(start, end, today()).is_err(),
                "{start}..{end}"
            );
        }
    }
}
