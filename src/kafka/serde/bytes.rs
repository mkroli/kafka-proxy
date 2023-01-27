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
use base64::engine::{GeneralPurpose, GeneralPurposeConfig};
use base64::{alphabet, Engine};
use uuid::Uuid;

const ENGINE: GeneralPurpose =
    GeneralPurpose::new(&alphabet::STANDARD, GeneralPurposeConfig::new());

fn deserialize_base64_byte_string(str: &str) -> Result<Vec<u8>> {
    let result = ENGINE.decode(str)?;
    Ok(result)
}

pub fn deserialize_string(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => Ok(Value::String(str)),
        v => bail!("Types don't match: String, {v}"),
    }
}

pub fn deserialize_bytes(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let bytes = deserialize_base64_byte_string(&str)?;
            Ok(Value::Bytes(bytes))
        }
        v => bail!("Types don't match: Bytes, {v}"),
    }
}

pub fn deserialize_fixed(size: usize, json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let bytes = deserialize_base64_byte_string(&str)?;
            if size != bytes.len() {
                bail!("Size of {str} doesn't match fixed size {size}");
            } else {
                Ok(Value::Fixed(size, bytes))
            }
        }
        v => bail!("Types don't match: Fixed, {v}"),
    }
}

pub fn deserialize_uuid(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let uuid = Uuid::parse_str(&str)?;
            Ok(Value::Uuid(uuid))
        }
        v => bail!("Types don't match: Uuid, {v}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;
    use uuid::Uuid;

    #[test]
    fn test_string() {
        assert_eq!(
            test("\"string\"", "\"test\"").unwrap(),
            Value::String("test".to_string())
        );
    }

    #[test]
    fn test_bytes() {
        assert_eq!(
            test(r#"{"type":"bytes"}"#, "\"dGVzdA==\"").unwrap(),
            Value::Bytes("test".into())
        );
    }

    #[test]
    fn test_fixed() {
        assert_eq!(
            test(
                r#"{"type":"fixed", "name":"test", "size":4}"#,
                "\"dGVzdA==\"",
            )
            .unwrap(),
            Value::Fixed(4, "test".into())
        );
    }

    #[test]
    fn test_uuid() {
        assert_eq!(
            test(
                r#"{"type":"string", "logicalType":"uuid"}"#,
                "\"550e8400-e29b-11d4-a716-446655440000\"",
            )
            .unwrap(),
            Value::Uuid(Uuid::parse_str("550e8400-e29b-11d4-a716-446655440000").unwrap())
        );
    }
}
