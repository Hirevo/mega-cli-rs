mod deserialize;
mod serialize;

use std::fmt;
use std::time::Duration;

pub use self::deserialize::*;
pub use self::serialize::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ParseDurationError;

impl fmt::Display for ParseDurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error parsing duration")
    }
}

impl std::error::Error for ParseDurationError {}

pub fn parse_duration(value: &str) -> Result<Duration, ParseDurationError> {
    let value = value.trim();
    let position = value.chars().take_while(|it| it.is_ascii_digit()).count();
    if position == 0 {
        return Err(ParseDurationError);
    };

    let (number_str, unit_str) = value.split_at(position);
    let Ok(number) = number_str.trim().parse::<u64>() else {
        return Err(ParseDurationError);
    };

    let duration = match unit_str.trim() {
        "ns" => Duration::from_nanos(number),
        "µs" => Duration::from_micros(number),
        "ms" => Duration::from_millis(number),
        "s" => Duration::from_secs(number),
        "min" => Duration::from_secs(number * 60),
        "h" => Duration::from_secs(number * 3600), // 60 * 60 == 3600
        _ => {
            return Err(ParseDurationError);
        }
    };

    Ok(duration)
}
