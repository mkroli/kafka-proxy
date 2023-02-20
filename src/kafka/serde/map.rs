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
use std::collections::HashMap;

pub fn deserialize(schema: &Schema, json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Object(m) => {
            let mut map = HashMap::with_capacity(m.len());
            for (key, value) in m {
                let value = crate::kafka::serde::deserialize(schema, value)?;
                map.insert(key, value);
            }
            Ok(Value::Map(map))
        }
        v => bail!("Types don't match: Map, {v}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_map() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), Value::Int(1));
        map.insert("b".to_string(), Value::Int(2));
        map.insert("c".to_string(), Value::Int(3));
        assert_eq!(
            test(
                &json!({"type":"map","values":"int"}),
                json!({"a":1,"b":2,"c":3}),
            )
            .unwrap(),
            Value::Map(map)
        );
    }
}
