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
use apache_avro::schema::RecordField;
use apache_avro::types::Value;

pub fn deserialize(fields: &Vec<RecordField>, json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Object(mut obj) => {
            let mut result_fields = Vec::with_capacity(fields.len());
            for field in fields {
                let json = obj
                    .remove(&field.name)
                    .or_else(|| field.default.clone())
                    .with_context(|| format!("Field not found: {}", field.name))?;
                let value = crate::kafka::serde::deserialize(&field.schema, json)?;
                result_fields.push((field.name.clone(), value));
            }
            Ok(Value::Record(result_fields))
        }
        _ => bail!("Types don't match: Record, {json}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;
    use serde_json::json;

    #[test]
    fn test_record() {
        let schema = json!({
            "type": "record",
            "name": "record",
            "fields": [
                {"name": "a", "type": "int"},
                {"name": "b", "type": "string"},
                {"name": "c", "type": "long", "default": 1}
            ]
        });
        assert_eq!(
            test(&schema, json!({"a": 1, "b": "2"})).unwrap(),
            Value::Record(vec!(
                ("a".to_string(), Value::Int(1)),
                ("b".to_string(), Value::String("2".to_string())),
                ("c".to_string(), Value::Long(1))
            ))
        );
    }
}
