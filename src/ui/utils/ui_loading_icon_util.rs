use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

/// takes single dots and combines them into a multi dot
const fn icon<const L: usize>(dots: [char; L]) -> char {
    let mut pattern: u8 = 0;

    let mut i = 0;
    while i < dots.len() {
        pattern |= match dots[i] {
            '⠁' => 0b0000_0001, // Dot 1
            '⠂' => 0b0000_0010, // Dot 2
            '⠄' => 0b0000_0100, // Dot 3
            '⠈' => 0b0000_1000, // Dot 4
            '⠐' => 0b0001_0000, // Dot 5
            '⠠' => 0b0010_0000, // Dot 6
            '⡀' => 0b0100_0000, // Dot 7
            '⢀' => 0b1000_0000, // Dot 8
            _ => panic!("invalid input"),
        };
        i += 1;
    }

    char::from_u32('⠀' as u32 + pattern as u32).unwrap()
}

// ⠁⠈
// ⠂⠐
// ⠄⠠
// ⡀⢀
pub const LOADING_ICONS_LEN: u8 = 8;
const FULL_ROTATION_SEC: f64 = 1.0 / 2.0;
const LOADING_ICONS: [char; LOADING_ICONS_LEN as usize] = [
    icon(['⠁','⠈','⠐']),
    icon(['⠈','⠐','⠠']),
    icon(['⠐','⠠','⢀']),
    icon(['⠠','⢀','⡀']),
    icon(['⢀','⡀','⠄']),
    icon(['⡀','⠄','⠂']),
    icon(['⠄','⠂','⠁']),
    icon(['⠂','⠁','⠈']),
];

pub fn loading_icon(interval: u8) -> char {
    LOADING_ICONS[interval as usize]
}

pub fn next_loading_rotation_interval(now: SystemTime) -> Duration {
    let epoch    = now.duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
    let interval = FULL_ROTATION_SEC / LOADING_ICONS_LEN as f64;
    let next     = interval - (epoch % interval);
    debug_assert!(next <= interval);
    Duration::from_secs_f64(next)
}
