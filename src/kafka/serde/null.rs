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

pub fn deserialize(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::Null => Ok(Value::Null),
        v => bail!("Types don't match: Null, {v}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;

    #[test]
    fn test_null() {
        assert_eq!(test("\"null\"", "null").unwrap(), Value::Null);
    }
}
