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
use crate::commands::Commands;
use crate::context::Context;
use crate::scripts::script::ResolveScript;

use crate::dc_tasks::DcTasksTrait;
use crate::subcommands::PassThru;
use m2::M2Recipe;

use crate::output_files::OutputFiles;
use crate::recipes::validate::ValidateRecipe;
use wp::WpRecipe;

pub mod m2;
pub mod recipe_kinds;
pub mod validate;
pub mod wp;

pub trait Recipe<'a, 'b>:
    Commands<'a, 'b> + PassThru + DcTasksTrait + OutputFiles + ResolveScript + ValidateRecipe
{
    fn default_help(&self, _ctx: &Context) -> Result<String, failure::Error> {
        Ok(String::from("default_help not implemented"))
    }
}

pub fn available_recipes<'a, 'b>() -> Vec<Box<dyn Recipe<'a, 'b>>> {
    vec![Box::new(M2Recipe), Box::new(WpRecipe)]
}
