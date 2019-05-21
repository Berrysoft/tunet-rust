use std::string;
use std::time;

pub fn format_flux(flux: u64) -> string::String {
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

pub fn format_duration(d: time::Duration) -> string::String {
    let mut total_sec = d.as_secs();
    let sec = total_sec % 60;
    total_sec /= 60;
    let min = total_sec % 60;
    total_sec /= 60;
    format!("{:02}:{:02}:{:02}", total_sec, min, sec)
}
