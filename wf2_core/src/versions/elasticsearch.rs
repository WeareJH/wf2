use serde::de;
use std::fmt;
use std::fmt::Formatter;


//TODO: Refactor to make more generic, so we can use for php versions too.
macro_rules! build_es_version {
($default:ident,[$( $key:ident => $value:expr ),*]) => {
    #[derive(Debug, Clone, PartialEq)]
    pub enum ELASTICSEARCH {
    $($key,)*
    }
    impl ELASTICSEARCH {
    pub fn get_image(&self) -> f64 {
        match self {
            $(
                ELASTICSEARCH::$key => $value,
            )*
        }
    }
    }

    impl Default for ELASTICSEARCH {
        fn default() -> Self {
            ELASTICSEARCH::$default
        }
    }


    impl std::fmt::Display for ELASTICSEARCH {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self)
        }
    }

    pub fn deserialize_elasticsearch<'de, D>(deserializer: D) -> Result<ELASTICSEARCH, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ELASTICSEARCHVisitor;

        impl<'de> de::Visitor<'de> for ELASTICSEARCHVisitor {
            type Value = ELASTICSEARCH;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                let versions = vec![$($value.to_string(),)*].iter().map(|val| format!("`{}`",val)).collect::<Vec<String>>().join(", ");
                formatter.write_fmt(format_args!("either {}",versions))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let versions = vec![$($value.to_string(),)*].join(", ");
                let r = match v {
                    $(num if num == $value => Ok(ELASTICSEARCH::$key),)*
                        _ => Err(format!("expected versions: {}",versions)),
                };
                r.map_err(E::custom)
            }
        }

        deserializer.deserialize_any(ELASTICSEARCHVisitor)
    }


}

}
build_es_version!(SevenSix,[SixEight => 6.8,SevenSix => 7.6]);
