use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::dc_tasks::M2Volumes;
use crate::recipes::m2::m2_vars::{M2Var, M2Vars};
use crate::recipes::m2::services::M2_ROOT;
use crate::services::Service;

use crate::services::node::NodeService;

pub struct M2NodeService;

impl Service<M2Vars> for M2NodeService {
    const NAME: &'static str = NodeService::NAME;
    const IMAGE: &'static str = NodeService::IMAGE;

    fn dc_service(&self, ctx: &Context, vars: &M2Vars) -> DcService {
        (NodeService)
            .dc_service(ctx, &())
            .set_working_dir(M2_ROOT)
            .set_init(true)
            .set_volumes(vec![format!("{}:{}:z", M2Volumes::APP, M2_ROOT)])
            .set_env_file(vec![vars.content[&M2Var::EnvFile].to_string()])
            .finish()
    }
}
