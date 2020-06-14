use crate::cmd::PassThruCmd;
use crate::commands::{CliCommand, Commands};
use crate::context::Context;

use crate::dc_tasks::DcTasksTrait;
use crate::dc_volume::DcVolume;
use crate::recipes::wp::pass_thru::WpPassThru;
use crate::recipes::wp::subcommands::{wp_recipe_global_subcommands, wp_recipe_subcommands};
use crate::recipes::Recipe;
use crate::scripts::script::ResolveScript;
use crate::task::Task;

use crate::output_files::OutputFiles;
use crate::recipes::validate::ValidateRecipe;
use crate::recipes::wp::services::WpServices;
use crate::services::Services;
use crate::subcommands::PassThru;
use volumes::get_volumes;

pub struct WpRecipe;

impl ValidateRecipe for WpRecipe {}

impl WpRecipe {
    const DEFAULT_DOMAIN: &'static str = "localhost:8080";
    pub fn ctx_domain(ctx: &Context) -> String {
        ctx.domains
            .get(0)
            .and_then(|d| {
                if d == "local.m2" {
                    None
                } else {
                    Some(d.clone())
                }
            })
            .unwrap_or_else(|| String::from(WpRecipe::DEFAULT_DOMAIN))
    }
    pub fn ctx_port(ctx: &Context) -> String {
        let d = WpRecipe::ctx_domain(&ctx);
        let _s = d.contains(':');
        if d.contains(':') {
            let s: String = d.split(':').skip(1).take(1).collect();
            return s;
        }
        String::from("80")
    }
}

impl DcTasksTrait for WpRecipe {
    fn volumes(&self, ctx: &Context) -> Vec<DcVolume> {
        get_volumes(ctx)
    }
    fn services(&self, ctx: &Context) -> Result<Box<dyn Services>, failure::Error> {
        let services = WpServices::from_ctx(ctx);
        Ok(Box::new(services))
    }
}

pub mod pass_thru;
pub mod services;
pub mod subcommands;
pub mod volumes;

impl<'a, 'b> Commands<'a, 'b> for WpRecipe {
    fn subcommands(&self, _ctx: &Context) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        wp_recipe_subcommands()
    }
    fn global_subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        wp_recipe_global_subcommands()
    }
}

impl<'a, 'b> Recipe<'a, 'b> for WpRecipe {
    fn default_help(&self, _ctx: &Context) -> Result<String, failure::Error> {
        unimplemented!()
    }
}

impl PassThru for WpRecipe {
    fn resolve(&self, ctx: &Context, cmd: &PassThruCmd) -> Option<Vec<Task>> {
        let dc_tasks = (WpRecipe).dc_tasks(&ctx);
        if let Err(e) = dc_tasks {
            return Some(Task::task_err_vec(e));
        }
        WpPassThru::resolve_cmd(
            &ctx,
            cmd.cmd.to_string(),
            &cmd.trailing,
            dc_tasks.expect("guarded above"),
        )
    }
    fn names(&self, _ctx: &Context) -> Vec<(String, String)> {
        WpPassThru::commands()
    }
}

impl OutputFiles for WpRecipe {}
impl ResolveScript for WpRecipe {}
