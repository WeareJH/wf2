use crate::context::Context;
use crate::dc_service::DcService;
use crate::recipes::m2::m2_vars::M2Vars;
use crate::recipes::m2::services::traefik::TraefikService;
use crate::recipes::m2::services::M2Service;
use std::fmt;

pub struct MailService;

impl MailService {
    pub const DOMAIN: &'static str = "mail.jh";
}

impl fmt::Display for MailService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "MailHog: https://{}", MailService::DOMAIN)
    }
}

impl M2Service for MailService {
    const NAME: &'static str = "mail";
    const IMAGE: &'static str = "mailhog/mailhog";

    fn dc_service(&self, ctx: &Context, _vars: &M2Vars) -> DcService {
        DcService::new(ctx.name(), Self::NAME, Self::IMAGE)
            .set_ports(vec!["1025"])
            .set_labels(TraefikService::host_entry_label(
                MailService::DOMAIN,
                8025_u32,
            ))
            .build()
    }
}
