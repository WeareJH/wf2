use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::M2Vars;
use crate::recipes::m2::services::M2Service;
use crate::recipes::m2::volumes::M2Volumes;

pub struct ElasticSearchService;

impl M2Service for ElasticSearchService {
    const NAME: &'static str = "elasticsearch";
    const IMAGE: &'static str = "wearejh/elasticsearch:5.6-m2";

    fn dc_service(&self, ctx: &Context, _vars: &M2Vars) -> DcService {
        DcService::new(ctx.name.clone(), Self::NAME, Self::IMAGE)
            .set_ports(vec!["9200:9200"])
            .set_volumes(vec![format!(
                "{}:/usr/share/elasticsearch/data",
                M2Volumes::ELASTICSEARCH
            )])
            .set_environment(vec!["discovery.type=single-node"])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .build()
    }
}
