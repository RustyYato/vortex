pub struct KeyDoesNotExistError;

impl<'de> serde::Deserialize<'de> for KeyDoesNotExistError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct KeyDoesNotExistErrorVisitor;

        impl<'de> serde::de::Visitor<'de> for KeyDoesNotExistErrorVisitor {
            type Value = KeyDoesNotExistError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a cas error")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "key does not exist" {
                    Ok(KeyDoesNotExistError)
                } else {
                    Err(E::invalid_value(
                        serde::de::Unexpected::Str(v),
                        &"key does not exist",
                    ))
                }
            }
        }

        deserializer.deserialize_str(KeyDoesNotExistErrorVisitor)
    }
}

pub struct CasError {
    pub actual: u32,
}

impl<'de> serde::Deserialize<'de> for CasError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CasErrorVisitor;

        impl<'de> serde::de::Visitor<'de> for CasErrorVisitor {
            type Value = CasError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a cas error")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let prev = v
                    .strip_prefix("current value ")
                    .and_then(|v| v.split_once(" is not "))
                    .map(|(prev, _)| prev)
                    .ok_or_else(|| E::custom("invalid cas error"))?;

                let actual = prev
                    .parse()
                    .map_err(|_| E::custom("case error previous must be an integer"))?;

                Ok(CasError { actual })
            }
        }

        deserializer.deserialize_str(CasErrorVisitor)
    }
}
