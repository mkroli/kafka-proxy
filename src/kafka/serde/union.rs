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

use anyhow::Context;
use anyhow::Result;
use apache_avro::schema::UnionSchema;
use apache_avro::types::Value;

pub fn deserialize(schema: &UnionSchema, json: serde_json::Value) -> Result<Value> {
    let mut result = None;
    for (index, variant) in schema.variants().iter().enumerate() {
        if let Ok(v) = crate::kafka::serde::deserialize(variant, json.clone()) {
            result = Some(Value::Union(index as u32, Box::new(v)));
            break;
        }
    }
    result.with_context(|| format!("No type matched: {schema:?}, {json}"))
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;
    use serde_json::json;

    #[test]
    fn test_union() {
        let schema = json!(["null", "int", "string"]);
        assert_eq!(
            test(&schema, json!(null)).unwrap(),
            Value::Union(0, Box::new(Value::Null))
        );
        assert_eq!(
            test(&schema, json!(123)).unwrap(),
            Value::Union(1, Box::new(Value::Int(123)))
        );
        assert_eq!(
            test(&schema, json!("test123")).unwrap(),
            Value::Union(2, Box::new(Value::String("test123".to_string())))
        );
    }
}
