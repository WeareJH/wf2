//!
//!
//!
use crate::file::File;
use crate::recipes::m2::output_files::auth::Auth;
use crate::recipes::m2::output_files::composer::Composer;
use crate::recipes::m2::services::M2RecipeOptions;
use crate::recipes::validate::ValidateRecipe;
use crate::{context::Context, recipes::Recipe, task::Task};

pub mod dc_tasks;
#[doc(hidden)]
pub mod m2_vars;
#[doc(hidden)]
pub mod output_files;
#[doc(hidden)]
pub mod pass_thru;
#[doc(hidden)]
pub mod resolve_script;
pub mod services;
pub mod subcommands;
#[doc(hidden)]
pub mod tasks;

///
/// PHP 7.1 + 7.2 + 7.3 Environments for use with Magento 2.
///
#[derive(Default)]
pub struct M2Recipe;

impl<'a, 'b> Recipe<'a, 'b> for M2Recipe {}

impl ValidateRecipe for M2Recipe {
    fn validate(&self, ctx: &Context) -> Task {
        let mut tasks = match (Composer::from_ctx(&ctx), Auth::from_ctx(&ctx)) {
            (Ok(c), Ok(a)) => vec![c.exists_task(), a.exists_task()],
            _ => vec![],
        };

        let attempt_pwa = ctx
            .options
            .as_ref()
            .map_or(false, |opts| opts["services"]["pwa"].is_mapping());

        if attempt_pwa {
            if let Err(e) = ctx.parse_options::<M2RecipeOptions>() {
                tasks.push(Task::notify_error(format!(
                    "PWA options are invalid\n{}",
                    e
                )));
            }
        }

        Task::Seq(tasks)
    }
}

#[cfg(test)]
mod tests {
    use crate::context::Context;
    use crate::dc_tasks::DcTasksTrait;
    use crate::recipes::m2::M2Recipe;
    use crate::services::pwa::PwaService;
    use crate::services::Service;

    #[test]
    fn test_get_services_with_pwa() {
        let ctx_yaml = r#"
        recipe: M2
        php_version: 7.2
        domains: [ example.m2 ]
        options:
          services:
            pwa:
              domains: [ example.pwa ]
              src_dir: ~/
        "#;
        let ctx = Context::new_from_str(ctx_yaml).expect("test");
        let m2 = (M2Recipe).services(&ctx).expect("test services");
        assert!(m2.service_by_name(PwaService::NAME).is_some());
    }
}
