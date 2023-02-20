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

use anyhow::{bail, Result};
use apache_avro::types::Value;
use apache_avro::Schema;

mod array;
mod boolean;
mod bytes;
mod datetime;
mod r#enum;
mod map;
mod null;
mod number;
mod record;
mod union;

fn deserialize(schema: &Schema, json: serde_json::Value) -> Result<Value> {
    let value = match schema {
        Schema::Null => null::deserialize(json)?,
        Schema::Boolean => boolean::deserialize(json)?,
        Schema::Int => number::deserialize_int(json)?,
        Schema::Long => number::deserialize_long(json)?,
        Schema::Float => number::deserialize_float(json)?,
        Schema::Double => number::deserialize_double(json)?,
        Schema::Bytes => bytes::deserialize_bytes(json)?,
        Schema::String => bytes::deserialize_string(json)?,
        Schema::Array(schema) => array::deserialize(schema, json)?,
        Schema::Map(schema) => map::deserialize(schema, json)?,
        Schema::Union(schema) => union::deserialize(schema, json)?,
        Schema::Record { fields, .. } => record::deserialize(fields, json)?,
        Schema::Enum { symbols, .. } => r#enum::deserialize(symbols, json)?,
        Schema::Fixed { size, .. } => bytes::deserialize_fixed(*size, json)?,
        Schema::Decimal { scale, .. } => number::deserialize_decimal(*scale as u32, json)?,
        Schema::Uuid => bytes::deserialize_uuid(json)?,
        Schema::Date => datetime::deserialize_date(json)?,
        Schema::TimeMillis => datetime::deserialize_time_millis(json)?,
        Schema::TimeMicros => datetime::deserialize_time_micros(json)?,
        Schema::TimestampMillis => datetime::deserialize_timestamp_millis(json)?,
        Schema::TimestampMicros => datetime::deserialize_timestamp_micros(json)?,
        Schema::Duration => bail!("Not implemented: Duration"),
        Schema::Ref { .. } => bail!("Not implemented: Ref"),
    };
    Ok(value)
}

pub fn deserialize_json(schema: &Schema, json: serde_json::Value) -> Result<Value> {
    deserialize(schema, json)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use apache_avro::types::Value;
    use apache_avro::Schema;
    use serde_json::json;

    use crate::kafka::serde::deserialize_json;

    pub fn test(tp: &serde_json::Value, json: serde_json::Value) -> Result<Value> {
        let schema = json!({
            "name": "value",
            "type": tp,
        });
        let schema = Schema::parse(&schema)?;
        let value = deserialize_json(&schema, json)?;
        Ok(value)
    }

    #[test]
    #[ignore]
    fn test_duration() {}

    #[test]
    #[ignore]
    fn test_ref() {}
}
