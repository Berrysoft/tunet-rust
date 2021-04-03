use chrono::{NaiveDate, NaiveDateTime};
use std::io::Write;
use std::time::Duration;
use termcolor::{Color, ColorSpec, StandardStream, WriteColor};
use tunet_rust::*;

pub trait FmtColor {
    fn fmt_color(&self, w: &mut StandardStream) -> Result<()> {
        self.fmt_color_aligned(w)
    }
    fn fmt_color_aligned(&self, w: &mut StandardStream) -> Result<()>;
}

impl FmtColor for Flux {
    fn fmt_color(&self, w: &mut StandardStream) -> Result<()> {
        let flux = self.0;
        let mut spec = ColorSpec::new();
        w.set_color(if flux == 0 {
            spec.set_fg(Some(Color::Cyan))
        } else if flux < 20_000_000_000 {
            spec.set_fg(Some(Color::Yellow)).set_bold(true)
        } else {
            spec.set_fg(Some(Color::Magenta)).set_bold(true)
        })?;
        write!(w, "{}", self)?;
        Ok(())
    }
    fn fmt_color_aligned(&self, w: &mut StandardStream) -> Result<()> {
        let flux = self.0;
        let mut spec = ColorSpec::new();
        w.set_color(if flux == 0 {
            spec.set_fg(Some(Color::Cyan))
        } else if flux < 2_000_000_000 {
            spec.set_fg(Some(Color::Yellow)).set_bold(true)
        } else {
            spec.set_fg(Some(Color::Magenta)).set_bold(true)
        })?;
        write!(w, "{:>8}", format!("{}", self))?;
        Ok(())
    }
}

impl FmtColor for Duration {
    fn fmt_color_aligned(&self, w: &mut StandardStream) -> Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        let mut total_sec = self.as_secs();
        let sec = total_sec % 60;
        total_sec /= 60;
        let min = total_sec % 60;
        total_sec /= 60;
        let h = total_sec % 24;
        total_sec /= 24;
        if total_sec > 0 {
            write!(w, "{}.{:02}:{:02}:{:02}", total_sec, h, min, sec)?;
        } else {
            write!(w, "{:02}:{:02}:{:02}", h, min, sec)?;
        }
        Ok(())
    }
}

impl FmtColor for Balance {
    fn fmt_color_aligned(&self, w: &mut StandardStream) -> Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
        write!(w, "{}", self)?;
        Ok(())
    }
}

static TUNET_DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
static TUNET_DATE_FORMAT: &str = "%Y-%m-%d";

impl FmtColor for NaiveDateTime {
    fn fmt_color_aligned(&self, w: &mut StandardStream) -> Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(w, "{:20}", self.format(TUNET_DATE_TIME_FORMAT))?;
        Ok(())
    }
}

impl FmtColor for NaiveDate {
    fn fmt_color_aligned(&self, w: &mut StandardStream) -> Result<()> {
        w.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        write!(w, "{:10}", self.format(TUNET_DATE_FORMAT))?;
        Ok(())
    }
}
