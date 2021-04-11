use std::fmt;
use std::fmt::Display;
use tunet_rust::*;

pub struct FmtDuration(pub Duration);

impl Display for FmtDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_sec = self.0.num_seconds();
        let (total_min, sec) = (total_sec / 60, total_sec % 60);
        let (total_h, min) = (total_min / 60, total_min % 60);
        let (day, h) = (total_h / 24, total_h % 24);
        let str = if day != 0 {
            format!("{}.{:02}:{:02}:{:02}", day, h, min, sec)
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
