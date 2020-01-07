use crate::dc_service::DcService;
use crate::dc_volume::DcVolume;
use std::collections::HashMap;

///
/// [`dc`] provides a struct that can be serialized to a docker-composer.yaml file.
///
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Dc {
    pub version: String,
    pub volumes: Option<HashMap<String, DcVolume>>,
    pub services: Option<HashMap<String, DcService>>,
}

impl Dc {
    const VERSION: &'static str = "3.7";

    pub fn new() -> Dc {
        Dc {
            version: String::from(Dc::VERSION),
            ..Dc::default()
        }
    }
    pub fn set_volumes(&mut self, volumes: &[DcVolume]) -> &mut Dc {
        let as_hashmap = volumes
            .iter()
            .map(|vol| (vol.display_name.clone(), vol.clone()))
            .collect::<HashMap<String, DcVolume>>();
        self.volumes = Some(as_hashmap);
        self
    }
    pub fn set_services(&mut self, services: &[DcService]) -> &mut Dc {
        let as_hashmap = services
            .iter()
            .map(|dc_service| (dc_service.name.to_string(), dc_service.clone()))
            .collect::<HashMap<String, DcService>>();
        self.services = Some(as_hashmap);
        self
    }
    pub fn build(&self) -> Dc {
        Dc { ..self.clone() }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_yaml::to_vec(self).expect("works")
    }
    pub fn service_names(&self) -> Option<Vec<String>> {
        self.services
            .as_ref()
            .map(|services| services.iter().map(|(key, _)| key.into()).collect())
    }
    pub fn service_img(&self) -> Vec<(String, String)> {
        self.services.as_ref().map_or(vec![], |services| {
            services
                .iter()
                .map(|(key, service)| (key.to_string(), service.image.clone()))
                .collect()
        })
    }
}
