#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct DcService {
    // This is used internally only
    #[serde(skip_serializing)]
    pub name: String,

    // required field
    pub container_name: String,

    // required field
    pub image: String,

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
    pub fn build(&self) -> DcService {
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
