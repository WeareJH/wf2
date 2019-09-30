use crate::scripts::script::Script;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
pub struct Scripts(pub HashMap<String, Script>);

impl Scripts {
    pub fn keys(&self) -> Vec<String> {
        self.0.keys().map(|x| x.to_owned()).collect()
    }
    pub fn pairs(&self) -> Vec<(String, String)> {
        self.0
            .iter()
            .map(|(name, script)| {
                (
                    name.to_owned(),
                    script.description.clone().unwrap_or(String::from("")),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::task::Task;

    #[test]
    fn test_scripts() {
        //language=yaml
        let yaml = r#"
      mage:
        steps:
          - sh: wf2 m c:f
          - dc: logs unison
          - run:
              service: node
              workdir: app/design/frontend/Bso/default
              user: www-data
              env:
                - NAME=kittie
                - API=key
              command: |
                yarn --production
                ./node_modules/.bin/cb prod:*
          - exec:
              service: php
              workdir: app/design/frontend/Bso/default
              user: www-data
              command: |
                yarn --production
                ./node_modules/.bin/cb prod:*

      fe:
        steps:
          - run:
              service: node
              workdir: app/design/frontend/Bso/default
              user: www-data
              command: |
                yarn --production
                ./node_modules/.bin/cb prod:*

      bundle:
        description: Build production assets locally for testing
        steps:
          - mage
          - fe
        "#;
        let scripts: Scripts = serde_yaml::from_str(yaml).expect("test");
        let cmd = scripts.0.get("mage").expect("test").to_owned();
        let cmd = cmd.set_dc_file(String::from("/users/shane/docker-compose.yml"));
        let ts: Vec<Task> = cmd.into();
        println!("{:#?}", ts);
    }
}
