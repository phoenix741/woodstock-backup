use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use utoipa::ToSchema;
use woodstock::utils::path::{mangle_buffer, unmangle_buffer};

/// BigInt scalar compatible avec le schéma NestJS
/// Implémentation: transport GraphQL en String, mapping interne vers u64.
#[derive(Clone, Copy, Debug, Default, ToSchema)]
#[schema(value_type = String, example = "9007199254740991", description = "BigInt transporté en chaîne pour compatibilité GraphQL/NestJS ; mappé en interne vers u64.")]
pub struct BigIntScalar(pub u64);

impl Serialize for BigIntScalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Sérialise en string pour maintenir la compatibilité avec NestJS
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for BigIntScalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<u64>()
            .map(BigIntScalar)
            .map_err(serde::de::Error::custom)
    }
}

#[Scalar(name = "BigInt")]
impl ScalarType for BigIntScalar {
    fn parse(value: async_graphql::Value) -> InputValueResult<Self> {
        match value {
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    Ok(BigIntScalar(u))
                } else {
                    Err(InputValueError::custom("BigInt expects unsigned integer"))
                }
            }
            Value::String(s) => {
                if s.is_empty() {
                    return Err(InputValueError::custom(
                        "The value cannot be converted from BigInt because it is empty string",
                    ));
                }
                s.parse::<u128>()
                    .map_err(|_| InputValueError::custom(format!("The value {s} cannot be converted to a BigInt because it is not an integer")))
                    .and_then(|v| {
                        if v <= u64::MAX as u128 { Ok(BigIntScalar(v as u64)) } else { Err(InputValueError::custom("BigInt out of range for u64")) }
                    })
            }
            _ => Err(InputValueError::custom(
                "BigInt cannot represent non-integer value",
            )),
        }
    }

    fn to_value(&self) -> Value {
        // Sérialise en string comme NestJS BigIntScalar
        Value::String(self.0.to_string())
    }
}

#[derive(Clone, Debug, Default, ToSchema)]
#[schema(value_type = String, description = "Buffer binaire encodé en chaîne via mangle_buffer/unmangle_buffer pour le transport GraphQL.")]
pub struct BufferScalar(pub Vec<u8>);

impl Serialize for BufferScalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Sérialise en utilisant mangle_buffer pour maintenir la cohérence
        serializer.serialize_str(&mangle_buffer(&self.0))
    }
}

impl<'de> Deserialize<'de> for BufferScalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(BufferScalar(unmangle_buffer(&s)))
    }
}

#[Scalar(name = "Buffer")]
impl ScalarType for BufferScalar {
    fn parse(value: async_graphql::Value) -> InputValueResult<Self> {
        match value {
            Value::String(s) => Ok(BufferScalar(unmangle_buffer(&s))),
            _ => Err(InputValueError::custom(
                "Buffer can't represent non-string value",
            )),
        }
    }

    fn to_value(&self) -> Value {
        // Sérialise en string comme NestJS BigIntScalar
        Value::String(mangle_buffer(&self.0))
    }
}
