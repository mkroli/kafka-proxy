/*
 * Copyright 2023 Michael Krolikowski
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use anyhow::Result;
use anyhow::{bail, Context};
use apache_avro::types::Value;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::ops::{AddAssign, DivAssign};

pub fn deserialize_int(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let int = dec
                .to_i32()
                .with_context(|| format!("{dec} cannot be represented as i32"))?;
            Ok(Value::Int(int))
        }
        v => bail!("Types don't match: Int, {v}"),
    }
}

pub fn deserialize_long(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let long = dec
                .to_i64()
                .with_context(|| format!("{dec} cannot be represented as i64"))?;
            Ok(Value::Long(long))
        }
        v => bail!("Types don't match: Long, {v}"),
    }
}

pub fn deserialize_float(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let float = dec
                .to_f32()
                .with_context(|| format!("{dec} cannot be represented as f32"))?;
            Ok(Value::Float(float))
        }
        v => bail!("Types don't match: Float, {v}"),
    }
}

pub fn deserialize_double(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let double = dec
                .to_f64()
                .with_context(|| format!("{dec} cannot be represented as f64"))?;
            Ok(Value::Double(double))
        }
        v => bail!("Types don't match: Double, {v}"),
    }
}

pub fn deserialize_decimal(scale: u32, json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Number(n) => {
            let mut dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            dec.div_assign(Decimal::new(1, scale));
            if dec.is_sign_negative() {
                dec.add_assign(Decimal::new(1, 0));
            }
            dec.rescale(0);
            let unpacked = dec.unpack();
            let mut bytes = Vec::with_capacity(16);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.push(0);
            bytes.append(&mut unpacked.hi.to_be_bytes().to_vec());
            bytes.append(&mut unpacked.mid.to_be_bytes().to_vec());
            bytes.append(&mut unpacked.lo.to_be_bytes().to_vec());
            if unpacked.negative {
                for item in &mut bytes {
                    *item = !*item;
                }
            }
            Ok(Value::Decimal(apache_avro::Decimal::from(bytes)))
        }
        v => bail!("Types don't match: Decimal, {v}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;
    use apache_avro::Decimal;
    use serde_json::json;

    #[test]
    fn test_int() {
        assert_eq!(test(&json!("int"), json!(123)).unwrap(), Value::Int(123));
    }

    #[test]
    fn test_long() {
        assert_eq!(test(&json!("long"), json!(123)).unwrap(), Value::Long(123));
    }

    #[test]
    fn test_float() {
        assert_eq!(
            test(&json!("float"), json!(123.123)).unwrap(),
            Value::Float(123.123)
        );
    }

    #[test]
    fn test_double() {
        assert_eq!(
            test(&json!("double"), json!(123.123)).unwrap(),
            Value::Double(123.123)
        );
    }

    #[test]
    fn test_decimal() {
        assert_eq!(
            test(
                &json!({"type": "bytes", "logicalType": "decimal", "precision": 9, "scale": 6}),
                json!(123.456789),
            )
            .unwrap(),
            Value::Decimal(Decimal::from(vec!(
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x5B,
                0xCD, 0x15
            )))
        );
    }
}
