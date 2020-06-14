use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct DcImageBuild {
    pub context: PathBuf,
    pub dockerfile: PathBuf,
    pub args: BTreeMap<String, String>,
}
