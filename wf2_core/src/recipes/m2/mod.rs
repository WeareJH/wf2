//!
//!
//!
use crate::commands::CliCommand;
use crate::dc::Dc;
use crate::file::File;
use crate::recipes::m2::resolve_script::resolve;
use crate::recipes::m2::subcommands::m2_recipe_global_subcommands;
use crate::recipes::m2::templates::auth::Auth;
use crate::recipes::m2::templates::composer::Composer;
use crate::scripts::script::Script;
use crate::{cmd::Cmd, context::Context, dc_tasks::DcTasks, recipes::Recipe, task::Task};
use clap::ArgMatches;
use m2_vars::{M2Vars, Vars};
use pass_thru::M2PassThru;
use services::get_services;
use subcommands::m2_recipe_subcommands;
use volumes::get_volumes;

#[doc(hidden)]
pub mod m2_runtime_env_file;
#[doc(hidden)]
pub mod m2_vars;
#[doc(hidden)]
pub mod pass_thru;
#[doc(hidden)]
pub mod resolve_script;
pub mod services;
pub mod subcommands;
#[doc(hidden)]
pub mod tasks;
#[doc(hidden)]
pub mod templates;
pub mod volumes;

///
/// PHP 7.1 + 7.2 + 7.3 Environments for use with Magento 2.
///
#[derive(Default)]
pub struct M2Recipe;

impl<'a, 'b> Recipe<'a, 'b> for M2Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        match cmd {
            Cmd::PassThrough { cmd, trailing } => M2PassThru::resolve_cmd(&ctx, cmd, trailing),
        }
    }
    fn subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        m2_recipe_subcommands()
    }
    fn global_subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        m2_recipe_global_subcommands()
    }
    fn pass_thru_commands(&self) -> Vec<(String, String)> {
        pass_thru::commands()
    }
    fn select_command(&self, input: (&str, Option<&ArgMatches<'a>>)) -> Option<Cmd> {
        Cmd::select_pass_thru(input)
    }
    fn resolve_script(&self, ctx: &Context, script: &Script) -> Option<Vec<Task>> {
        resolve(ctx, script)
    }
    fn default_help(&self, _ctx: &Context) -> Result<String, failure::Error> {
        unimplemented!()
    }
    fn validate(&self, ctx: &Context) -> Task {
        match (Composer::from_ctx(&ctx), Auth::from_ctx(&ctx)) {
            (Ok(c), Ok(a)) => Task::Seq(vec![c.exists_task(), a.exists_task()]),
            _ => Task::Noop,
        }
    }
}

impl M2Recipe {
    pub fn dc_tasks(ctx: &Context) -> Result<DcTasks, failure::Error> {
        let vars = M2Vars::from_ctx(&ctx)?;
        let dc = Dc::new()
            .set_volumes(&get_volumes(&ctx))
            .set_services(&get_services(&vars, &ctx))
            .build();

        Ok(DcTasks::from_ctx(&ctx, dc.to_bytes()))
    }

    pub fn dc(ctx: &Context) -> Result<Dc, failure::Error> {
        let vars = M2Vars::from_ctx(&ctx)?;
        Ok(Dc::new()
            .set_volumes(&get_volumes(&ctx))
            .set_services(&get_services(&vars, &ctx))
            .build())
    }

    pub fn dc_and_tasks(ctx: &Context) -> Result<(Dc, DcTasks), failure::Error> {
        let dc = M2Recipe::dc(&ctx)?;
        let bytes = dc.to_bytes();
        let tasks = DcTasks::from_ctx(&ctx, bytes);

        Ok((dc, tasks))
    }
}
