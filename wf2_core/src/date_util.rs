use chrono::prelude::*;

use regex::Regex;

///
/// DateUtil - Convienience methods for setting times/dates
/// from input tha needs to be sanitized or changed
///
/// ## Examples
///
/// Set the date + time from &str inputs (which means it can fail in many ways)
///
/// ```rust
/// # fn main() -> Result<(), failure::Error> {
/// # use chrono::prelude::*;
/// # use wf2_core::date_util::DateUtil;
/// let d = DateUtil::new_local().set_date_time("2014-02-01", "08:08:00")?;
/// assert_eq!((d.year(), d.month(), d.day()), (2014, 2, 1));
/// assert_eq!((d.hour(), d.minute(), d.second()), (8, 8, 0));
/// # Ok(())
/// # }
/// ```
///
/// Set just the time
///
/// ```rust
/// # fn main() -> Result<(), failure::Error> {
/// # use chrono::prelude::*;
/// # use chrono::TimeZone;
/// # use wf2_core::date_util::DateUtil;
/// let now = Local.ymd(2020, 1, 1).and_hms(8, 0, 0);
/// let d = DateUtil::new(now).set_time("12:15:00")?;
/// assert_eq!((d.year(), d.month(), d.day()), (2020, 1, 1));
/// assert_eq!((d.hour(), d.minute(), d.second()), (12, 15, 0));
/// # Ok(())
/// # }
/// ```
///
/// Set just the date
///
/// ```rust
/// # fn main() -> Result<(), failure::Error> {
/// # use chrono::prelude::*;
/// # use chrono::TimeZone;
/// # use wf2_core::date_util::DateUtil;
/// let now = Local.ymd(2020, 1, 1).and_hms(8, 0, 0);
/// let d = DateUtil::new(now).set_date("2020-02-02")?;
/// assert_eq!((d.year(), d.month(), d.day()), (2020, 2, 2));
/// assert_eq!((d.hour(), d.minute(), d.second()), (8, 0, 0));
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone)]
pub struct DateUtil {
    pub date: DateTime<Local>,
}

impl DateUtil {
    pub fn new(d: DateTime<Local>) -> DateUtil {
        DateUtil { date: d }
    }

    pub fn new_local() -> DateUtil {
        DateUtil { date: Local::now() }
    }

    ///
    /// Convenience method for setting the date + time in 1 go.
    ///
    /// Note errors will happen left-to-right and will fail fast, so
    /// it's possible that 1 input will error multiple times when
    /// each piece is corrected.
    ///
    pub fn set_date_time(
        &self,
        date_str: &str,
        time_str: &str,
    ) -> Result<DateTime<Local>, failure::Error> {
        DateUtil::_set_date(self.date, date_str).and_then(|d| DateUtil::_set_time(d, time_str))
    }

    ///
    /// Set the time element of a [`DateTime`] using a string input.
    ///
    /// Each segment is set individually to allow fine-grained
    /// errors such as 'invalid year' etc.
    ///
    pub fn set_date(&self, date_str: &str) -> Result<DateTime<Local>, failure::Error> {
        DateUtil::_set_date(self.date, date_str)
    }

    fn _set_date(date: DateTime<Local>, date_str: &str) -> Result<DateTime<Local>, failure::Error> {
        // the date regex
        let date_re = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").unwrap();

        let caps = date_re
            .captures(date_str)
            .ok_or(DateError::InvalidFormat(date_str.into()))?;

        let y = caps[1].parse::<i32>()?;
        let m = caps[2].parse::<u32>()?;
        let d = caps[3].parse::<u32>()?;

        let date = date.with_year(y).ok_or(DateError::InvalidYear(y))?;
        let date = date.with_month(m).ok_or(DateError::InvalidMonth(m))?;
        let date = date.with_day(d).ok_or(DateError::InvalidDay(d))?;

        Ok(date)
    }

    ///
    /// Set the time element of a [`DateTime`] using a string input.
    ///
    /// Each segment is set individually to allow fine-grained
    /// errors such as 'invalid minute' etc.
    ///
    pub fn set_time(&self, time_str: &str) -> Result<DateTime<Local>, failure::Error> {
        DateUtil::_set_time(self.date, time_str)
    }
    pub fn _set_time(
        date: DateTime<Local>,
        time_str: &str,
    ) -> Result<DateTime<Local>, failure::Error> {
        // the time regex
        let time_re = Regex::new(r"^(\d{2}):(\d{2}):(\d{2})$").unwrap();

        // try to match
        let caps = time_re
            .captures(time_str)
            .ok_or(TimeError::InvalidFormat(time_str.into()))?;

        let h = caps[1].parse::<u32>()?;
        let m = caps[2].parse::<u32>()?;
        let s = caps[3].parse::<u32>()?;

        let date = date.with_hour(h).ok_or(TimeError::InvalidHour(h))?;
        let date = date.with_minute(m).ok_or(TimeError::InvalidMinute(m))?;
        let date = date.with_second(s).ok_or(TimeError::InvalidSecond(s))?;

        Ok(date)
    }
}

#[derive(Debug, Fail)]
enum DateError {
    #[fail(
        display = "Invalid date format provided `{}`, should be YYYY-MM-DD",
        _0
    )]
    InvalidFormat(String),
    #[fail(display = "Invalid year provided `{}`", _0)]
    InvalidYear(i32),
    #[fail(display = "Invalid month provided `{}`", _0)]
    InvalidMonth(u32),
    #[fail(display = "Invalid day provided `{}`", _0)]
    InvalidDay(u32),
}

#[derive(Debug, Fail)]
enum TimeError {
    #[fail(display = "Invalid time format provided `{}`, should be HH:MM:SS", _0)]
    InvalidFormat(String),
    #[fail(display = "Invalid hour provided `{}`", _0)]
    InvalidHour(u32),
    #[fail(display = "Invalid minutes provided `{}`", _0)]
    InvalidMinute(u32),
    #[fail(display = "Invalid seconds provided `{}`", _0)]
    InvalidSecond(u32),
}
