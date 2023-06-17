use std::fmt;
use std::time::Duration;

use serde::de::{self, Deserializer, Visitor};

/// Deserializes either a number or a string representing a human-readable duration into a [`Duration`].
///
/// The string format supported is roughly (expressed as a regular expression):
/// `^\s*(?P<number>\d+)\s*(?P<unit>ns|µs|ms|s|min|h)\s*$`
pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationVisitor;

    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a positive integer number (as milliseconds), or a string containing a positive integer number followed by a time unit")
        }

        fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Duration::from_millis(value))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(value.as_str())
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            parse_duration(self, value)
        }

        fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let Ok(value) = u64::try_from(value) else {
                return Err(de::Error::invalid_value(de::Unexpected::Signed(i64::from(value)), &self));
            };
            self.visit_u64(value)
        }
    }

    deserializer.deserialize_any(DurationVisitor)
}

/// Same as `deserialize`, but parses into an `Option` instead, allowing the field to be missing.
pub fn deserialize_opt<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    struct DurationOptVisitor;

    impl<'de> Visitor<'de> for DurationOptVisitor {
        type Value = Option<Duration>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a positive integer number (as milliseconds), or a string containing a positive integer number followed by a time unit")
        }

        fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_u64(u64::from(value))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(Duration::from_millis(value)))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(value.as_str())
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            parse_duration(self, value).map(Some)
        }

        fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_i64(i64::from(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let Ok(value) = u64::try_from(value) else {
                return Err(de::Error::invalid_value(de::Unexpected::Signed(i64::from(value)), &self));
            };
            self.visit_u64(value)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_any(self)
        }
    }

    deserializer.deserialize_any(DurationOptVisitor)
}

fn parse_duration<'de, V, E>(visitor: V, value: &str) -> Result<Duration, E>
where
    V: Visitor<'de>,
    E: de::Error,
{
    let value = value.trim();
    let position = value.chars().take_while(|it| it.is_ascii_digit()).count();
    if position == 0 {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(value),
            &visitor,
        ));
    };

    let (number_str, unit_str) = value.split_at(position);
    let Ok(number) = number_str.trim().parse::<u64>() else {
        return Err(de::Error::invalid_value(
            de::Unexpected::Str(number_str.trim()),
            &"a positive integer number parsable into a `u64`",
        ));
    };

    let duration = match unit_str.trim() {
        "ns" => Duration::from_nanos(number),
        "µs" => Duration::from_micros(number),
        "ms" => Duration::from_millis(number),
        "s" => Duration::from_secs(number),
        "min" => Duration::from_secs(number * 60),
        "h" => Duration::from_secs(number * 3600), // 60 * 60 == 3600
        unit_str => {
            return Err(de::Error::invalid_value(
                de::Unexpected::Str(unit_str),
                &"a valid time unit (`ns`, `µs`, `ms`, `s`, `min`, or `h`)",
            ));
        }
    };

    Ok(duration)
}
