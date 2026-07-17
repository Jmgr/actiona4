use std::{fmt, time::Duration};

use const_default::ConstDefault;
use derive_more::Deref;
use macros::Parameter;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, Visitor},
};

#[derive(Clone, Copy, Debug, Default, Deref, Eq, Ord, PartialEq, PartialOrd)]
pub struct DurationValue(Duration);

impl DurationValue {
    pub const ZERO: Self = Self(Duration::ZERO);

    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }
}

impl fmt::Display for DurationValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        humantime::format_duration(self.0).fmt(f)
    }
}

impl From<Duration> for DurationValue {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

impl From<DurationValue> for Duration {
    fn from(duration: DurationValue) -> Self {
        duration.0
    }
}

impl Serialize for DurationValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            humantime::format_duration(self.0)
                .to_string()
                .serialize(serializer)
        } else {
            (self.0.as_secs(), self.0.subsec_nanos()).serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for DurationValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(DurationValueVisitor)
        } else {
            let (seconds, nanoseconds) = <(u64, u32)>::deserialize(deserializer)?;
            Ok(Self(Duration::new(seconds, nanoseconds)))
        }
    }
}

struct DurationValueVisitor;

impl<'de> Visitor<'de> for DurationValueVisitor {
    type Value = DurationValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "a duration string, a millisecond number, or a { secs, nanos } duration object",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        duration_from_str(value).map(DurationValue)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        duration_from_milliseconds(value as f64).map(DurationValue)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        duration_from_milliseconds(value as f64).map(DurationValue)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        duration_from_milliseconds(value).map(DurationValue)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut secs = None;
        let mut nanos = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "secs" => secs = Some(map.next_value()?),
                "nanos" => nanos = Some(map.next_value()?),
                _ => {
                    let _ = map.next_value::<de::IgnoredAny>()?;
                }
            }
        }

        let secs = secs.ok_or_else(|| de::Error::missing_field("secs"))?;
        let nanos = nanos.ok_or_else(|| de::Error::missing_field("nanos"))?;

        Ok(DurationValue(Duration::new(secs, nanos)))
    }
}

fn duration_from_str<E>(value: &str) -> Result<Duration, E>
where
    E: de::Error,
{
    humantime::parse_duration(value)
        .map_err(|err| E::custom(format!("failed to parse duration `{value}`: {err}")))
}

fn duration_from_milliseconds<E>(milliseconds: f64) -> Result<Duration, E>
where
    E: de::Error,
{
    Duration::try_from_secs_f64(milliseconds / 1_000.0)
        .map_err(|_| E::custom("duration number must be finite and greater than or equal to 0"))
}

#[derive(ConstDefault, Debug, Parameter)]
#[parameter(storage = DurationValue)]
pub struct DurationParameter;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rstest::rstest;
    use serde_json::json;

    use super::DurationValue;

    #[rstest]
    #[case(Duration::ZERO, "0s")]
    #[case(Duration::from_millis(250), "250ms")]
    #[case(Duration::new(1, 500_000_000), "1s 500ms")]
    fn serializes_as_humantime_string(#[case] duration: Duration, #[case] expected: &str) {
        let value = DurationValue::new(duration);

        let json = serde_json::to_value(value).expect("serialize duration");

        assert_eq!(json, json!(expected));
    }

    #[rstest]
    #[case(json!("0s"), Duration::ZERO)]
    #[case(json!("250ms"), Duration::from_millis(250))]
    #[case(json!("1.5s"), Duration::from_millis(1_500))]
    #[case(json!(250), Duration::from_millis(250))]
    #[case(json!(1.5), Duration::from_micros(1_500))]
    #[case(json!({ "secs": 1, "nanos": 500_000_000 }), Duration::new(1, 500_000_000))]
    #[case(
        json!({ "secs": 1, "nanos": 500_000_000, "ignored": true }),
        Duration::new(1, 500_000_000)
    )]
    fn deserializes_supported_wire_formats(
        #[case] json: serde_json::Value,
        #[case] expected: Duration,
    ) {
        let duration: DurationValue = serde_json::from_value(json).expect("deserialize duration");

        assert_eq!(duration, DurationValue::new(expected));
    }

    #[rstest]
    #[case(json!("10q"))]
    #[case(json!(-1))]
    #[case(json!(f64::NAN))]
    #[case(json!({ "secs": 1 }))]
    #[case(json!({ "nanos": 1 }))]
    fn rejects_invalid_wire_formats(#[case] json: serde_json::Value) {
        let result = serde_json::from_value::<DurationValue>(json);

        assert!(result.is_err());
    }

    #[test]
    fn converts_to_and_from_std_duration() {
        let duration = Duration::from_millis(250);
        let value = DurationValue::from(duration);

        assert_eq!(*value, duration);
        assert_eq!(Duration::from(value), duration);
    }

    #[test]
    fn roundtrips_as_seconds_and_nanoseconds_in_binary_formats() {
        let duration = DurationValue::new(Duration::new(1, 500_000_000));

        let bytes = postcard::to_allocvec(&duration).expect("serialize duration");
        let decoded: DurationValue = postcard::from_bytes(&bytes).expect("deserialize duration");

        assert_eq!(decoded, duration);
    }
}
