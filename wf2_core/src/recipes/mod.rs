use crate::scripts::script::Script;
use crate::{cmd::Cmd, context::Context, task::Task};
use clap::{App, ArgMatches};

pub mod m2;
pub mod m2_contrib;
pub mod recipe_kinds;

pub trait Recipe<'a, 'b> {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>>;
    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        vec![]
    }
    fn pass_thru_commands(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn select_command(&self, input: (&str, Option<&ArgMatches<'a>>)) -> Option<Cmd>;
    fn resolve_script(&self, ctx: &Context, script: &Script) -> Option<Vec<Task>>;
}

#[derive(Clone)]
pub struct RecipeTemplate {
    pub bytes: Vec<u8>,
}
