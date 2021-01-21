#[derive(Clone, Debug, Deserialize)]
pub struct Stores(pub Vec<Store>);

///
/// Deref and DerefMut traits let us use the Stores struct
/// as if it were a Vec<Store>.
///
impl std::ops::Deref for Stores {
    type Target = Vec<Store>;
    fn deref(&self) -> &Vec<Store> {
        &self.0
    }
}

impl std::ops::DerefMut for Stores {
    fn deref_mut(&mut self) -> &mut Vec<Store> {
        &mut self.0
    }
}

///
/// Store struct represents a given store in M2
///
#[derive(Clone, Debug, Deserialize)]
pub struct Store {
    pub path_prefix: String,
    pub mage_run_code: String,
    pub mage_run_type: String,
}

///
/// Each store can be used to generate a partial nginx configuration file
/// by replacing placeholders from a given template.
///
impl Store {
    pub fn process_template(&self, template: &str) -> String {
        template
            .replace("{{path_prefix}}", &self.path_prefix.trim_end_matches('/'))
            .replace("{{mage_run_code}}", &self.mage_run_code)
            .replace("{{mage_run_type}}", &self.mage_run_type)
    }
}
