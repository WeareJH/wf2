#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DcVolume {
    pub name: String,

    // internal use only
    #[serde(skip_serializing)]
    pub display_name: String,
}

impl DcVolume {
    pub fn new(ctx_name: impl Into<String>, name: impl Into<String>) -> DcVolume {
        let name: String = name.into();
        DcVolume {
            name: format!("wf2__{}__{}", ctx_name.into(), name),
            display_name: name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_dc_volume() {
        let dcv = DcVolume::new("acme", "app-db");
        assert_eq!(dcv.name, "wf2__acme__app-db");
        assert_eq!(dcv.display_name, "app-db");
    }
}
