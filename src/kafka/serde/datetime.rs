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
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use rust_decimal::prelude::ToPrimitive;

fn deserialize_datetime(str: &str) -> Result<DateTime<FixedOffset>> {
    let date_time = str.parse::<DateTime<FixedOffset>>()?;
    Ok(date_time)
}

pub fn deserialize_date(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let date_time = deserialize_datetime(&str)?;
            let duration = date_time
                .date_naive()
                .signed_duration_since(NaiveDate::default());
            Ok(Value::Date(duration.num_days() as i32))
        }
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let date = dec
                .to_i32()
                .with_context(|| format!("{dec} cannot be represented as i32"))?;
            Ok(Value::Date(date))
        }
        v => bail!("Types don't match: Date, {v}"),
    }
}

pub fn deserialize_time_millis(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let date_time = deserialize_datetime(&str)?;
            let duration = date_time.time().signed_duration_since(NaiveTime::default());
            Ok(Value::TimeMillis(duration.num_milliseconds() as i32))
        }
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let time = dec
                .to_i32()
                .with_context(|| format!("{dec} cannot be represented as i32"))?;
            Ok(Value::TimeMillis(time))
        }
        v => bail!("Types don't match: TimeMillis, {v}"),
    }
}

pub fn deserialize_time_micros(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let date_time = deserialize_datetime(&str)?;
            let duration = date_time.time().signed_duration_since(NaiveTime::default());
            let micros = duration
                .num_microseconds()
                .with_context(|| format!("{duration} microseconds overflow"))?;
            Ok(Value::TimeMicros(micros))
        }
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let time = dec
                .to_i64()
                .with_context(|| format!("{dec} cannot be represented as i64"))?;
            Ok(Value::TimeMicros(time))
        }
        v => bail!("Types don't match: TimeMicros, {v}"),
    }
}

pub fn deserialize_timestamp_millis(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let date_time = deserialize_datetime(&str)?;
            Ok(Value::TimestampMillis(date_time.timestamp_millis()))
        }
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let timestamp = dec
                .to_i64()
                .with_context(|| format!("{dec} cannot be represented as i64"))?;
            Ok(Value::TimestampMillis(timestamp))
        }
        v => bail!("Types don't match: TimestampMillis, {v}"),
    }
}

pub fn deserialize_timestamp_micros(json: serde_json::Value) -> Result<Value> {
    match json {
        serde_json::Value::String(str) => {
            let date_time = deserialize_datetime(&str)?;
            Ok(Value::TimestampMicros(date_time.timestamp_micros()))
        }
        serde_json::Value::Number(n) => {
            let dec = rust_decimal::serde::arbitrary_precision::deserialize(n)?;
            let timestamp = dec
                .to_i64()
                .with_context(|| format!("{dec} cannot be represented as i64"))?;
            Ok(Value::TimestampMicros(timestamp))
        }
        v => bail!("Types don't match: TimestampMillis, {v}"),
    }
}

#[cfg(test)]
mod test {
    use crate::kafka::serde::tests::test;
    use apache_avro::types::Value;
    use serde_json::json;

    #[test]
    fn test_date() {
        assert_eq!(
            test(
                &json!({"type":"int", "logicalType":"date"}),
                json!("2001-02-03T12:34:56.789Z"),
            )
            .unwrap(),
            Value::Date(11356)
        )
    }

    #[test]
    fn test_time_millis() {
        assert_eq!(
            test(
                &json!({"type":"int", "logicalType":"time-millis"}),
                json!("2001-02-03T12:34:56.789Z"),
            )
            .unwrap(),
            Value::TimeMillis(45296789)
        );
    }

    #[test]
    fn test_time_micros() {
        assert_eq!(
            test(
                &json!({"type":"long", "logicalType":"time-micros"}),
                json!("2001-02-03T12:34:56.789Z"),
            )
            .unwrap(),
            Value::TimeMicros(45296789000)
        );
    }

    #[test]
    fn test_timestamp_millis() {
        assert_eq!(
            test(
                &json!({"type":"long", "logicalType":"timestamp-millis"}),
                json!("2001-02-03T12:34:56.789Z"),
            )
            .unwrap(),
            Value::TimestampMillis(981203696789)
        );
    }

    #[test]
    fn test_timestamp_micros() {
        assert_eq!(
            test(
                &json!({"type":"long", "logicalType":"timestamp-micros"}),
                json!("2001-02-03T12:34:56.789Z"),
            )
            .unwrap(),
            Value::TimestampMicros(981203696789000)
        );
    }
}
