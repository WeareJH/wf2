use serde::de;
use std::fmt;

#[derive(Debug, Clone)]
pub enum PHP {
    SevenOne,
    SevenTwo,
}

impl Default for PHP {
    fn default() -> Self {
        PHP::SevenTwo
    }
}

///
/// Helpers for deserializing direct from yaml
///
pub fn deserialize_php<'de, D>(deserializer: D) -> Result<PHP, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct PHPVisitor;

    impl<'de> de::Visitor<'de> for PHPVisitor {
        type Value = PHP;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("either `7.1` or `7.2`")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let r = match v {
                7.1 => Ok(PHP::SevenOne),
                7.2 => Ok(PHP::SevenTwo),
                _ => Err("expected either 7.1 or 7.2"),
            };
            r.map_err(E::custom)
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let r = match v {
                "7.1" => Ok(PHP::SevenOne),
                "7.2" => Ok(PHP::SevenTwo),
                _ => Err("expected either 7.1 or 7.2"),
            };
            r.map_err(E::custom)
        }
    }

    deserializer.deserialize_any(PHPVisitor)
}
