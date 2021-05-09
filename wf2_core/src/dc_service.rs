use crate::dc_image_build::DcImageBuild;
use crate::dc_service_network::DcServiceNetwork;
use std::collections::HashMap;

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct DcService {
    // This is used internally only
    #[serde(skip_serializing)]
    pub name: String,

    // required field
    pub container_name: String,

    // required field
    pub image: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub build: Option<DcImageBuild>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub init: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<HashMap<String, DcServiceNetwork>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub privileged: Option<bool>,
}

impl DcService {
    pub fn new(
        ctx_name: impl Into<String>,
        name: impl Into<String>,
        image: impl Into<String>,
    ) -> DcService {
        let name: String = name.into();
        DcService {
            name: name.clone(),
            container_name: format!("wf2__{}__{}", ctx_name.into(), name),
            image: image.into(),
            ..DcService::default()
        }
    }
    pub fn set_volumes(&mut self, volumes: Vec<impl Into<String>>) -> &mut Self {
        self.volumes = Some(volumes.into_iter().map(|x| x.into()).collect());
        self
    }
    pub fn add_volumes(&mut self, volumes: Vec<impl Into<String>>) -> &mut Self {
        let next_volumes = volumes
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<String>>();
        let mut prev_volumes = self.volumes.clone().unwrap_or_default();
        prev_volumes.extend(next_volumes);
        self.volumes = Some(prev_volumes);
        self
    }
    pub fn set_ports(&mut self, ports: Vec<impl Into<String>>) -> &mut Self {
        self.ports = Some(ports.into_iter().map(|x| x.into()).collect());
        self
    }
    pub fn set_command(&mut self, command: impl Into<String>) -> &mut Self {
        self.command = Some(command.into());
        self
    }
    pub fn set_labels(&mut self, labels: Vec<impl Into<String>>) -> &mut Self {
        self.labels = Some(labels.into_iter().map(|x| x.into()).collect());
        self
    }
    pub fn set_env_file(&mut self, env_file: Vec<impl Into<String>>) -> &mut Self {
        self.env_file = Some(env_file.into_iter().map(|x| x.into()).collect());
        self
    }
    pub fn set_depends_on(&mut self, depends_on: Vec<impl Into<String>>) -> &mut Self {
        self.depends_on = Some(depends_on.into_iter().map(|x| x.into()).collect());
        self
    }
    pub fn add_depends_on(&mut self, depends_on: Vec<impl Into<String>>) -> &mut Self {
        let next_depends_on = depends_on
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<String>>();
        let mut prev_depends_on = self.depends_on.clone().unwrap_or_default();
        prev_depends_on.extend(next_depends_on);
        self.depends_on = Some(prev_depends_on);
        self
    }
    pub fn set_environment(&mut self, environment: Vec<impl Into<String>>) -> &mut Self {
        self.environment = Some(environment.into_iter().map(|x| x.into()).collect());
        self
    }
    pub fn set_restart(&mut self, restart: impl Into<String>) -> &mut Self {
        self.restart = Some(restart.into());
        self
    }
    pub fn set_working_dir(&mut self, working_dir: impl Into<String>) -> &mut Self {
        self.working_dir = Some(working_dir.into());
        self
    }
    pub fn set_init(&mut self, init: bool) -> &mut Self {
        self.init = Some(init);
        self
    }
    pub fn set_container_name(
        &mut self,
        ctx_name: impl Into<String>,
        name: impl Into<String>,
    ) -> &mut Self {
        self.container_name = format!("wf2__{}__{}", ctx_name.into(), name.into());
        self
    }
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }
    pub fn set_image(&mut self, image: impl Into<String>) -> &mut Self {
        self.image = image.into();
        self
    }
    pub fn set_build(&mut self, build: DcImageBuild) -> &mut Self {
        self.build = Some(build);
        self
    }
    pub fn set_network(&mut self, name: impl Into<String>, network: DcServiceNetwork) -> &mut Self {
        if let Some(prev) = self.networks.as_mut() {
            prev.insert(name.into(), network);
        } else {
            let mut hm = HashMap::new();
            hm.insert(name.into(), network);
            self.networks = Some(hm)
        }
        self
    }
    pub fn set_privileged(&mut self, privileged: bool) -> &mut Self {
        self.privileged = Some(privileged);
        self
    }
    pub fn finish(&self) -> DcService {
        DcService { ..self.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dc_service() {
        let dcs = DcService::new("acme", "php", "wearejh/php:7.1");
        assert_eq!(dcs.name, "php");
        assert_eq!(dcs.container_name, "wf2__acme__php");
        assert_eq!(dcs.image, "wearejh/php:7.1");
    }
}
