use serde::ser::Serializer;
use chrono::{DateTime, Utc};

// "Date is expected to be in ISO 8601 format yyyy-MM-ddTHH:mm:ss.fffffff-HH:MM"
pub fn serialize_datetime<S>(value: &DateTime<Utc>, ser: S) -> Result<S::Ok, S::Error> where S: Serializer {
  let as_str = format!("{}", value.format("%Y-%m-%dT%H:%M:%S.0000000-00:00"));
  ser.serialize_str(&as_str)
}