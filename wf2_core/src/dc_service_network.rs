#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct DcServiceNetwork {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
}

impl DcServiceNetwork {
    pub fn with_aliases(aliases: Vec<impl Into<String>>) -> Self {
        DcServiceNetwork {
            aliases: Some(aliases.into_iter().map(|x| x.into()).collect()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dc_service::DcService;

    #[test]
    fn test_dc_service_network() {
        let actual_dc = DcService::new("acme", "php", "traefik:latest")
            .set_network(
                "default",
                DcServiceNetwork::with_aliases(vec![String::from("acme.m2")]),
            )
            .finish();

        let expected = r#"

            name: "php"
            container_name: wf2__acme__php
            image: "traefik:latest"
            networks:
              default:
                aliases:
                  - acme.m2

        "#;

        let expected_dc: DcService = serde_yaml::from_str(expected).expect("teest yaml");
        assert_eq!(actual_dc, expected_dc);
    }
}
