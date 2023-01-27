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

pub fn deserialize(symbols: &Vec<String>, json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let idx = symbols
                .iter()
                .position(|s| s.eq(&str))
                .with_context(|| format!("{str} not found in {symbols:?}"))?;
            Ok(Value::Enum(idx as u32, str))
        }
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let idx = dec
                .to_u32()
                .with_context(|| format!("{dec} cannot be represented as u32"))?;
            let symbol = symbols
                .get(idx as usize)
                .with_context(|| format!("Index {idx} not found in symbols"))?;
            Ok(Value::Enum(idx, symbol.to_string()))
        }
        v => bail!("Types don't match: Enum, {v}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;

    const SCHEMA: &str = r#"{"type":"enum", "name": "Test", "symbols":["A", "B", "C"]}"#;

    #[test]
    fn test_enum_index() {
        assert_eq!(test(SCHEMA, "0").unwrap(), Value::Enum(0, "A".to_string()));
        assert_eq!(test(SCHEMA, "1").unwrap(), Value::Enum(1, "B".to_string()));
        assert_eq!(test(SCHEMA, "2").unwrap(), Value::Enum(2, "C".to_string()));
    }

    #[test]
    fn test_enum_symbol() {
        assert_eq!(
            test(SCHEMA, "\"A\"").unwrap(),
            Value::Enum(0, "A".to_string())
        );
        assert_eq!(
            test(SCHEMA, "\"B\"").unwrap(),
            Value::Enum(1, "B".to_string())
        );
        assert_eq!(
            test(SCHEMA, "\"C\"").unwrap(),
            Value::Enum(2, "C".to_string())
        );
    }
}
