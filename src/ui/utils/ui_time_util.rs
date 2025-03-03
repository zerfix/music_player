use std::fmt::Write;
use std::time::Duration;

pub fn render_duration(output: &mut String, duration: Duration) {
    let duration_sec = duration.as_secs();
    let seconds = duration_sec % 60;
    let minutes = duration_sec / 60 % 60;
    let hours   = duration_sec / 60 / 60;
    match hours > 0 {
        false => write!(output,       "{:02}:{:02}",        minutes, seconds).unwrap(),
        true  => write!(output, "{:02}:{:02}:{:02}", hours, minutes, seconds).unwrap(),
    };
}
