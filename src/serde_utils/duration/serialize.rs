use std::time::Duration;

use serde::ser::Serializer;

/// Serializes a [`Duration`] into a string representing a human-readable duration.
///
/// The string format supported is roughly (expressed as a regular expression):
/// `^\s*(?P<number>\d+)\s*(?P<unit>ns|µs|ms|s|min|h)\s*$`
pub fn serialize<S>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    const UNITS: &[(&str, u128)] = &[
        ("h", 3_600_000_000_000),
        ("min", 60_000_000_000),
        ("s", 1_000_000_000),
        ("ms", 1_000_000),
        ("µs", 1_000),
        ("ns", 0),
    ];

    let value = value.as_nanos();
    let (unit, threshold) = UNITS
        .iter()
        .copied()
        .find(|(_, threshold)| value >= *threshold)
        .expect("could not find a suitable time unit (unreachable)");

    let serialized = format!("{0} {unit}", value / threshold);
    serializer.serialize_str(&serialized)
}

/// Same as `serialize`, but serializes an `Option<Duration>` instead, allowing the field to be missing.
pub fn serialize_opt<S>(value: &Option<Duration>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(value) => serialize(value, serializer),
        None => serializer.serialize_none(),
    }
}
