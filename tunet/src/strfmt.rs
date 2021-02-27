use ansi_term::{ANSIString, Color};
use chrono::{NaiveDate, NaiveDateTime};
use std::time;

pub fn format_flux(flux: u64) -> String {
    let mut f = flux as f64;
    if f < 1000.0 {
        return format!("{} B", f);
    }
    f /= 1000.0;
    if f < 1000.0 {
        return format!("{:.2} K", f);
    }
    f /= 1000.0;
    if f < 1000.0 {
        return format!("{:.2} M", f);
    }
    f /= 1000.0;
    return format!("{:.2} G", f);
}

pub fn colored_flux(flux: u64, total: bool, right_aligned: bool) -> ANSIString<'static> {
    let f = if right_aligned {
        format!("{:>8}", format_flux(flux))
    } else {
        format_flux(flux)
    };
    if flux == 0 {
        Color::Blue.normal().paint(f)
    } else if flux < if total { 20_000_000_000 } else { 2_000_000_000 } {
        Color::Yellow.bold().paint(f)
    } else {
        Color::Purple.bold().paint(f)
    }
}

pub fn format_duration(d: time::Duration) -> String {
    let mut total_sec = d.as_secs();
    let sec = total_sec % 60;
    total_sec /= 60;
    let min = total_sec % 60;
    total_sec /= 60;
    let h = total_sec % 24;
    total_sec /= 24;
    if total_sec > 0 {
        format!("{}.{:02}:{:02}:{:02}", total_sec, h, min, sec)
    } else {
        format!("{:02}:{:02}:{:02}", h, min, sec)
    }
}

pub fn colored_duration(d: time::Duration) -> ANSIString<'static> {
    Color::Green.normal().paint(format_duration(d))
}

pub fn format_currency(c: f64) -> String {
    format!("¥{:.2}", c)
}

pub fn colored_currency(c: f64) -> ANSIString<'static> {
    Color::Yellow.normal().paint(format_currency(c))
}

const TUNET_DATE_TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";
const TUNET_DATE_FORMAT: &'static str = "%Y-%m-%d";

pub fn format_date_time(t: NaiveDateTime) -> String {
    t.format(TUNET_DATE_TIME_FORMAT).to_string()
}

pub fn colored_date_time(t: NaiveDateTime) -> ANSIString<'static> {
    Color::Green
        .normal()
        .paint(format!("{:20}", format_date_time(t)))
}

pub fn format_date(t: NaiveDate) -> String {
    t.format(TUNET_DATE_FORMAT).to_string()
}

pub fn colored_date(t: NaiveDate) -> ANSIString<'static> {
    Color::Green
        .normal()
        .paint(format!("{:10}", format_date(t)))
}
