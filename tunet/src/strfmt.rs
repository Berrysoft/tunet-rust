use chrono::{NaiveDate, NaiveDateTime};
use std::fmt;
use std::fmt::Display;
use std::time::Duration;

pub struct FmtDuration(pub Duration);

impl Display for FmtDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut total_sec = self.0.as_secs();
        let sec = total_sec % 60;
        total_sec /= 60;
        let min = total_sec % 60;
        total_sec /= 60;
        let h = total_sec % 24;
        total_sec /= 24;
        let str = if total_sec > 0 {
            format!("{}.{:02}:{:02}:{:02}", total_sec, h, min, sec)
        } else {
            format!("{:02}:{:02}:{:02}", h, min, sec)
        };
        f.pad(&str)
    }
}

static TUNET_DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
static TUNET_DATE_FORMAT: &str = "%Y-%m-%d";

pub struct FmtDateTime(pub NaiveDateTime);

impl Display for FmtDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.format(TUNET_DATE_TIME_FORMAT).fmt(f)
    }
}

pub struct FmtDate(pub NaiveDate);

impl Display for FmtDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.format(TUNET_DATE_FORMAT).fmt(f)
    }
}
