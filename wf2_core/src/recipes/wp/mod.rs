use crate::cmd::Cmd;
use crate::commands::CliCommand;
use crate::context::Context;
use crate::dc::Dc;
use crate::dc_tasks::DcTasks;
use crate::recipes::wp::pass_thru::WpPassThru;
use crate::recipes::wp::subcommands::{wp_recipe_global_subcommands, wp_recipe_subcommands};
use crate::recipes::Recipe;
use crate::scripts::script::Script;
use crate::task::Task;
use clap::ArgMatches;
use services::get_services;
use volumes::get_volumes;

pub struct WpRecipe;

impl WpRecipe {
    const DEFAULT_DOMAIN: &'static str = "localhost:8080";
    pub fn dc_tasks(ctx: &Context) -> Result<DcTasks, failure::Error> {
        let dc = Dc::new()
            .set_volumes(&get_volumes(&ctx))
            .set_services(&get_services(&ctx))
            .build();

        Ok(DcTasks::from_ctx(&ctx, dc.to_bytes()))
    }
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

pub mod pass_thru;
pub mod services;
pub mod subcommands;
pub mod volumes;

impl<'a, 'b> Recipe<'a, 'b> for WpRecipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        let dc_tasks = WpRecipe::dc_tasks(&ctx);
        if let Err(e) = dc_tasks {
            return Some(Task::task_err_vec(e));
        }
        match cmd {
            Cmd::PassThrough { cmd, trailing } => {
                WpPassThru::resolve_cmd(&ctx, cmd, trailing, dc_tasks.expect("guarded above"))
            }
        }
    }

    fn subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        wp_recipe_subcommands()
    }

    fn global_subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        wp_recipe_global_subcommands()
    }
    fn pass_thru_commands(&self) -> Vec<(String, String)> {
        WpPassThru::commands()
    }
    fn select_command(&self, input: (&str, Option<&ArgMatches<'a>>)) -> Option<Cmd> {
        Cmd::select_pass_thru(input)
    }

    fn resolve_script(&self, _ctx: &Context, _script: &Script) -> Option<Vec<Task>> {
        None
    }

    fn default_help(&self, _ctx: &Context) -> Result<String, failure::Error> {
        unimplemented!()
    }
}
