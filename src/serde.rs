pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

pub mod d_seconds_f64 {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(d: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(d.as_secs_f64())
    }

    pub fn deserialize<'de, D>(d: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let seconds: f64 = Deserialize::deserialize(d)?;
        Ok(Duration::from_secs_f64(seconds))
    }
}
