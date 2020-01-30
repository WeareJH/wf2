//!
//! Recipes in `wf2` define how a particular project
//! will run.
//!
//! The CLI will adapt based on the current project's 'recipe'
//! meaning you'll only see commands relevant to the project you
//! run the command in.
//!
//! ## Examples
//!
//! If your `wf2.yml` file contains the following...
//!
//! ```
//! # use wf2_core::context::Context;
//! # let input = r#"
//! recipe: M2
//! # "#;
//! # let _ctx: Context = serde_yaml::from_str(input).expect("Can parse m2 yaml example");
//! ```
//!
//! Then you'll only see content relevant to the [Magento 2](m2/index.html),
//! and the same thing for the [Wordpress Recipe](wp/index.html)
//!
//! ```
//! # use wf2_core::context::Context;
//! # let input = r#"
//! recipe: Wp
//! # "#;
//! # let _ctx: Context = serde_yaml::from_str(input).expect("Can parse wp yaml example");
//! ```
//!
//! ## Recipes
//!
//! - [Magento 2](m2/index.html)
//! - [Wordpress](wp/index.html)
//!
use crate::commands::CliCommand;
use crate::scripts::script::Script;
use crate::{cmd::Cmd, context::Context, task::Task};
use clap::ArgMatches;
use m2::M2Recipe;
use wp::WpRecipe;

pub mod m2;
pub mod recipe_kinds;
pub mod shared;
pub mod wp;

pub trait Recipe<'a, 'b> {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>>;
    fn subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        vec![]
    }
    fn global_subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        vec![]
    }
    fn pass_thru_commands(&self) -> Vec<(String, String)> {
        vec![]
    }
    fn select_command(&self, input: (&str, Option<&ArgMatches<'a>>)) -> Option<Cmd>;
    fn resolve_script(&self, ctx: &Context, script: &Script) -> Option<Vec<Task>>;
    fn default_help(&self, ctx: &Context) -> Result<String, failure::Error>;
    fn validate(&self, _ctx: &Context) -> Task {
        Task::Noop
    }
}

#[derive(Clone)]
pub struct RecipeTemplate {
    pub bytes: Vec<u8>,
}

pub fn available_recipes<'a, 'b>() -> Vec<Box<dyn Recipe<'a, 'b>>> {
    vec![Box::new(M2Recipe), Box::new(WpRecipe)]
}
