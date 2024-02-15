use num_bigint::BigInt;
use num_traits::FromPrimitive;

use crate::runtime;

use super::Value;

impl TryFrom<&Value> for BigInt {
    type Error = runtime::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let num = match value {
            Value::Number(num) => BigInt::from_f64(*num).unwrap(),
            Value::Boolean(value) => {
                if *value {
                    BigInt::from(1)
                } else {
                    BigInt::from(0)
                }
            }
            Value::Undefined => {
                return Err(runtime::Error::Type(
                    "Cannot convert undefined to BigInt".to_string(),
                ))
            }
            Value::BigInt(value) => value.clone(),
            Value::String(value) => value
                .parse()
                .map_err(|_| runtime::Error::Type("Cannot convert string to BigInt".to_string()))?,
            Value::Object(value) => {
                let err = match value {
                    Some(_) => runtime::Error::Type("Cannot convert object to BigInt".to_string()), // TODO this isn't right
                    None => runtime::Error::Type("Cannot convert null to BigInt".to_string()),
                };
                return Err(err);
            }
            Value::Symbol(_) => {
                return Err(runtime::Error::Type(
                    "Cannot convert Symbol to BigInt".to_string(),
                ))
            }
        };

        Ok(num)
    }
}
