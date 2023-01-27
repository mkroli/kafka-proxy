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

use anyhow::bail;
use anyhow::Result;
use apache_avro::types::Value;
use apache_avro::Schema;

pub fn deserialize(schema: &Schema, json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Array(arr) => {
            let mut result = Vec::with_capacity(arr.len());
            for v in arr {
                let val = crate::kafka::serde::Deserializer::deserialize(schema, v)?;
                result.push(val);
            }
            Ok(Value::Array(result))
        }
        _ => bail!("Types don't match: Array, {json}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;

    #[test]
    fn test_array() {
        assert_eq!(
            test(r#"{"type":"array", "items":"int"}"#, "[1, 2, 3]").unwrap(),
            Value::Array(vec!(Value::Int(1), Value::Int(2), Value::Int(3)))
        );
    }
}
