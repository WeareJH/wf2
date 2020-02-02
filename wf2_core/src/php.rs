use serde::de;
use std::fmt;

///
/// The PHP version used throughout this project.
///
/// This can be set within your `wf2.yml` file
///
/// ```yaml
/// php_version: 7.1
/// ```
///
#[derive(Debug, Clone, PartialEq)]
pub enum PHP {
    SevenOne,
    SevenTwo,
    SevenThree,
    SevenFour,
}

impl Default for PHP {
    fn default() -> Self {
        PHP::SevenThree
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
            formatter.write_str("either `7.1`, `7.2`, `7.3` or `7.4`")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let r = match v {
                num if num == 7.1 => Ok(PHP::SevenOne),
                num if num == 7.2 => Ok(PHP::SevenTwo),
                num if num == 7.3 => Ok(PHP::SevenThree),
                num if num == 7.4 => Ok(PHP::SevenFour),
                _ => Err("expected either 7.1, 7.2, 7.3 or 7.4"),
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
                "7.3" => Ok(PHP::SevenThree),
                "7.4" => Ok(PHP::SevenFour),
                _ => Err("expected either 7.1, 7.2, 7.3 or 7.4"),
            };
            r.map_err(E::custom)
        }
    }

    deserializer.deserialize_any(PHPVisitor)
}
