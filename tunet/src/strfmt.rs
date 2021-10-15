use std::fmt;
use std::fmt::Display;
use tunet_rust::*;

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
