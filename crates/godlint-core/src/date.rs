use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

const SECONDS_PER_DAY: u64 = 86_400;

const DAYS_FROM_YEAR_ZERO_ERA: i64 = 719_468;

const DAYS_PER_ERA: i64 = 146_097;

const MONTH_LENGTHS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Date {
    year: i32,
    month: u32,
    day: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DateError {
    Malformed,
    OutOfRange,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, DateError> {
        if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) {
            return Err(DateError::OutOfRange);
        }

        Ok(Self { year, month, day })
    }

    pub fn parse(text: &str) -> Result<Self, DateError> {
        let (year, month, day) = fields(text)?;

        Self::new(year, month, day)
    }

    pub fn today() -> Option<Self> {
        let seconds = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
        let days = i64::try_from(seconds / SECONDS_PER_DAY).ok()?;

        Self::from_days_since_epoch(days)
    }

    fn from_days_since_epoch(days: i64) -> Option<Self> {
        let shifted = days + DAYS_FROM_YEAR_ZERO_ERA;
        let era = if shifted >= 0 {
            shifted
        } else {
            shifted - (DAYS_PER_ERA - 1)
        } / DAYS_PER_ERA;
        let day_of_era = shifted - era * DAYS_PER_ERA;
        let year_of_era =
            (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month = if shifted_month < 10 {
            shifted_month + 3
        } else {
            shifted_month - 9
        };
        let year = year_of_era + era * 400 + i64::from(month <= 2);

        Self::new(
            i32::try_from(year).ok()?,
            u32::try_from(month).ok()?,
            u32::try_from(day).ok()?,
        )
        .ok()
    }
}

fn fields(text: &str) -> Result<(i32, u32, u32), DateError> {
    let bytes = text.as_bytes();

    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(DateError::Malformed);
    }

    Ok((
        digits(&text[..4])?,
        digits(&text[5..7])?,
        digits(&text[8..10])?,
    ))
}

fn digits<T: std::str::FromStr>(text: &str) -> Result<T, DateError> {
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(DateError::Malformed);
    }

    text.parse().map_err(|_| DateError::Malformed)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 2 && is_leap_year(year) {
        return 29;
    }

    MONTH_LENGTHS
        .get(usize::try_from(month).unwrap_or(0).wrapping_sub(1))
        .copied()
        .unwrap_or(0)
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

impl fmt::Display for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => write!(formatter, "date must be written YYYY-MM-DD"),
            Self::OutOfRange => write!(formatter, "date does not exist in the calendar"),
        }
    }
}

impl std::error::Error for DateError {}
