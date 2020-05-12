use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::M2Vars;
use crate::recipes::m2::services::M2Service;
use crate::recipes::m2::volumes::M2Volumes;

pub struct ElasticSearchService;

impl ElasticSearchService {
    const VOLUME_DATA: &'static str = "/usr/share/elasticsearch/data";
}

impl M2Service for ElasticSearchService {
    const NAME: &'static str = "elasticsearch";
    const IMAGE: &'static str = "wearejh/elasticsearch:7.6-m2";

    fn dc_service(&self, ctx: &Context, _vars: &M2Vars) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_ports(vec!["9200:9200"])
            .set_volumes(vec![format!(
                "{}:{}",
                M2Volumes::ELASTICSEARCH,
                ElasticSearchService::VOLUME_DATA
            )])
            .set_environment(vec!["discovery.type=single-node"])
            .set_labels(vec![Self::TRAEFIK_DISABLE_LABEL.to_string()])
            .build()
    }
}
