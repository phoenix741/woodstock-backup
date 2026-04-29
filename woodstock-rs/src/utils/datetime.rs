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

/// Deserialize a `u64` from either a native YAML integer or a string (e.g. `"4277787426816"`).
///
/// This is required for backward compatibility with files written by the former Node.js code,
/// which serialized `BigInt` values using the `!big` YAML tag, producing strings like
/// `!big '4277787426816'`. After the `!big` tag is stripped (or ignored), the value appears to
/// `serde_yaml_ng` as a plain string rather than an integer.
pub fn deserialize_u64_or_string<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    match Input::deserialize(deserializer)? {
        Input::Str(s) => s.parse::<u64>().map_err(de::Error::custom),
        Input::U64(n) => Ok(n),
        Input::I64(n) => u64::try_from(n).map_err(de::Error::custom),
    }
}
