use chrono::{DateTime, Local, TimeZone};
use serde::{de, Deserialize, Deserializer};

/// Deserialize a DateTime<Local> from either an RFC3339 string or an epoch number (seconds or milliseconds).
pub fn deserialize_local_datetime<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    match Input::deserialize(deserializer)? {
        Input::Str(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Local))
            .map_err(de::Error::custom),
        Input::I64(n) => from_epoch::<D::Error>(n),
        Input::U64(n) => from_epoch_u64::<D::Error>(n),
    }
}

/// Like `deserialize_local_datetime` but for `Option<DateTime<Local>>`, handling missing fields.
pub fn deserialize_option_local_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Local>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<Input>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(Input::Str(s)) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Local))
            .map(Some)
            .map_err(de::Error::custom),
        Some(Input::I64(n)) => from_epoch::<D::Error>(n).map(Some),
        Some(Input::U64(n)) => from_epoch_u64::<D::Error>(n).map(Some),
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Input {
    Str(String),
    I64(i64),
    U64(u64),
}

// Heuristic: absolute value >= 1e12 -> milliseconds, else seconds.
fn from_epoch<E: de::Error>(n: i64) -> Result<DateTime<Local>, E> {
    if n.unsigned_abs() >= 1_000_000_000_000 {
        let secs = n / 1000;
        let subms = (n % 1000).unsigned_abs() as u32;
        Local
            .timestamp_opt(secs, subms * 1_000_000)
            .single()
            .ok_or_else(|| E::custom("invalid epoch milliseconds"))
    } else {
        Local
            .timestamp_opt(n, 0)
            .single()
            .ok_or_else(|| E::custom("invalid epoch seconds"))
    }
}

fn from_epoch_u64<E: de::Error>(n: u64) -> Result<DateTime<Local>, E> {
    if n >= 1_000_000_000_000 {
        let secs = (n / 1000) as i64;
        let subms = (n % 1000) as u32;
        Local
            .timestamp_opt(secs, subms * 1_000_000)
            .single()
            .ok_or_else(|| E::custom("invalid epoch milliseconds"))
    } else {
        Local
            .timestamp_opt(n as i64, 0)
            .single()
            .ok_or_else(|| E::custom("invalid epoch seconds"))
    }
}

/// Deserialize a `u64` from either a native YAML integer, a plain string (`"4277787426816"`),
/// or a YAML-tagged scalar (`!big '4277787426816'`) as written by the former Node.js code.
///
/// The `#[serde(untagged)]` enum approach cannot handle YAML tagged values because
/// `serde_yaml_ng` presents them as enum input, which untagged enums reject.
/// A manual `Visitor` is required to implement `visit_enum` for this case.
pub fn deserialize_u64_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(U64OrStringVisitor)
}

struct U64OrStringVisitor;

impl<'de> de::Visitor<'de> for U64OrStringVisitor {
    type Value = u64;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a u64, a string containing a u64, or a YAML tagged scalar (!big '...')")
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<u64, E> {
        Ok(v)
    }

    fn visit_u128<E: de::Error>(self, v: u128) -> Result<u64, E> {
        u64::try_from(v).map_err(de::Error::custom)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<u64, E> {
        u64::try_from(v).map_err(de::Error::custom)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<u64, E> {
        v.parse::<u64>().map_err(de::Error::custom)
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<u64, E> {
        v.parse::<u64>().map_err(de::Error::custom)
    }

    /// Handles YAML tagged scalars such as `!big '4277787426816'`.
    /// `serde_yaml_ng` presents them as an enum: the tag name is the variant,
    /// and the scalar content is the newtype payload.
    fn visit_enum<A>(self, data: A) -> Result<u64, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        use de::VariantAccess;
        // Discard the tag name (e.g. "big"); extract and recursively parse the inner value.
        let (_, variant) = data.variant::<de::IgnoredAny>()?;
        variant.newtype_variant_seed(U64OrStringSeed)
    }
}

struct U64OrStringSeed;

impl<'de> de::DeserializeSeed<'de> for U64OrStringSeed {
    type Value = u64;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<u64, D::Error> {
        deserializer.deserialize_any(U64OrStringVisitor)
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;

    #[derive(Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_u64_or_string")]
        value: u64,
    }

    #[test]
    fn test_u64_native() {
        let yaml = "value: 4277787426816";
        let t: TestStruct = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(t.value, 4_277_787_426_816);
    }

    #[test]
    fn test_u64_plain_string() {
        let yaml = "value: '4277787426816'";
        let t: TestStruct = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(t.value, 4_277_787_426_816);
    }

    #[test]
    fn test_u64_big_tag() {
        // Format produced by Node.js js-yaml with BigInt custom type
        let yaml = "value: !big '4277787426816'";
        let t: TestStruct = serde_yaml_ng::from_str(yaml).unwrap();
        assert_eq!(t.value, 4_277_787_426_816);
    }
}
