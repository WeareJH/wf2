use crate::context::Context;
use crate::file::File;
use crate::recipes::m2::multi_store::Stores;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct NginxM2 {
    file_path: PathBuf,
    server_name: String,
    stores: Option<Stores>,
}

impl File<NginxM2> for NginxM2 {
    const DESCRIPTION: &'static str = "Writes the nginx m2 conf file";
    const HOST_OUTPUT_PATH: &'static str = "nginx/sites/site.conf";

    fn from_ctx(ctx: &Context) -> Result<NginxM2, failure::Error> {
        Ok(NginxM2 {
            file_path: ctx.output_file_path(Self::HOST_OUTPUT_PATH),
            server_name: ctx.domains().join(" "),
            stores: ctx.stores.clone(),
        })
    }

    fn file_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    fn bytes(&self) -> Vec<u8> {
        //build the multistore nginx config based on the stores found in the context object.
        let partial = include_str!("m2_store_partial.conf");
        let stores_config = match &self.stores {
            Some(ss) => ss.iter().fold(String::from(""), |acc, store| {
                format!("{}{}", acc, store.process_template(partial))
            }),
            None => String::from(""),
        };

        include_str!("m2.conf")
            .replace("{{m2_server_name}}", &self.server_name)
            .replace("{{m2_multistore}}", &stores_config)
            .bytes()
            .collect()
    }
}
