#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::error::Error;

use godlint_core::date::{Date, DateError};

fn parsed(text: &str) -> Date {
    Date::parse(text).unwrap_or_else(|error| panic!("parses {text}: {error}"))
}

#[test]
fn reads_a_calendar_date() {
    assert_eq!(parsed("2026-07-28").to_string(), "2026-07-28");
    assert_eq!(parsed("0001-01-01").to_string(), "0001-01-01");
}

#[test]
fn orders_dates_chronologically() {
    assert!(parsed("2026-07-28") < parsed("2026-07-29"));
    assert!(parsed("2026-07-28") < parsed("2026-08-01"));
    assert!(parsed("2026-12-31") < parsed("2027-01-01"));
    assert_eq!(parsed("2026-07-28"), parsed("2026-07-28"));
}

#[test]
fn rejects_a_malformed_date() {
    for text in [
        "2026-7-28",
        "2026/07/28",
        "26-07-28",
        "2026-07-28T00:00:00",
        "",
        "tomorrow",
        "+026-07-28",
        "2026-07-2x",
    ] {
        assert_eq!(
            Date::parse(text),
            Err(DateError::Malformed),
            "{text} should be malformed"
        );
    }
}

#[test]
fn rejects_a_date_that_is_not_in_the_calendar() {
    for text in ["2026-13-01", "2026-00-01", "2026-01-32", "2026-01-00"] {
        assert_eq!(
            Date::parse(text),
            Err(DateError::OutOfRange),
            "{text} should be out of range"
        );
    }
}

#[test]
fn knows_which_februaries_have_twenty_nine_days() {
    assert!(Date::parse("2024-02-29").is_ok());
    assert!(Date::parse("2000-02-29").is_ok());
    assert_eq!(Date::parse("2023-02-29"), Err(DateError::OutOfRange));
    assert_eq!(Date::parse("1900-02-29"), Err(DateError::OutOfRange));
    assert_eq!(Date::parse("2100-02-29"), Err(DateError::OutOfRange));
}

#[test]
fn builds_a_date_from_its_parts() {
    assert_eq!(Date::new(2026, 7, 28), Ok(parsed("2026-07-28")));
    assert_eq!(Date::new(2026, 13, 1), Err(DateError::OutOfRange));
    assert_eq!(Date::new(2026, 2, 30), Err(DateError::OutOfRange));
}

#[test]
fn reads_the_current_date_from_the_clock() {
    let today = Date::today().unwrap_or_else(|| panic!("reads the clock"));

    assert!(
        today > parsed("2020-01-01"),
        "{today} predates this project"
    );
    assert!(today < parsed("2200-01-01"), "{today} is implausibly late");
}

#[test]
fn describes_each_failure() {
    assert!(
        DateError::Malformed.to_string().contains("YYYY-MM-DD"),
        "a malformed date should name the format it wanted"
    );
    assert!(DateError::OutOfRange.to_string().contains("calendar"));
    assert!(DateError::Malformed.source().is_none());
}
