//!
//! Update the images used in a recipe.
//!
//! Recipes are made up of [services](../services/index.html) - and for each service
//! there's a corresponding image that's usually published on docker hub.
//!
//! For example, the PHP image used in this recipe might be
//! something from this page [`wearejh/php`](https://hub.docker.com/r/wearejh/php/tags) - and when
//! that's updated you could manually just run `docker pull wearejh/php:7.2-m2`
//!
//! But this command, `update-images` will do the hard work for you. If you don't provide
//! any parameters it will update ALL images, but if you just wanted to update a single one, or
//! a few, then you reference them by service name.
//!
//! ## Why would an image be updated?
//!
//! Images are typically updated when there's a bug fix, performance improvement or when they
//! include changes that make certain `wf2` features possible.
//!
//! # Example: update the image used by the php service only
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 update-images php
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # assert_eq!(commands.len(), 1)
//! ```
//!
//! # Example: update the image used by the php + varnish services
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 update-images php varnish
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # assert_eq!(commands.len(), 2)
//! ```
//!
//! # Example: update ALL images
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 update-images
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # assert_eq!(commands.len(), 13)
//! ```
//!
//! ## How do I find the service names in this recipe?
//!
//! If you're not sure which service uses which image, take a look at
//! the [list-images](../list_images/index.html) command.
//!
//!
use crate::commands::CliCommand;
use crate::context::Context;
use crate::dc::Dc;
use crate::recipes::m2::M2Recipe;
use crate::task::Task;
use clap::{App, Arg, ArgMatches};
use structopt::StructOpt;

#[doc_link::doc_link("/recipes/m2/subcommands/update_images")]
pub struct M2UpdateImages;

impl M2UpdateImages {
    const NAME: &'static str = "update-images";
    const ABOUT: &'static str = "[m2] Update images used in the current recipe by service name";
}

#[derive(StructOpt, Debug)]
struct Opts {
    services: Vec<String>,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2UpdateImages {
    fn name(&self) -> String {
        String::from(M2UpdateImages::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let opts: Opts = matches.map(Opts::from_clap).expect("guarded by Clap");
        let dc = M2Recipe::dc(&ctx);
        match dc {
            Ok(dc) => Some(update_images(&dc, opts.services)),
            Err(e) => Some(Task::task_err_vec(e)),
        }
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2UpdateImages::NAME)
            .about(M2UpdateImages::ABOUT)
            .after_help(M2UpdateImages::DOC_LINK)
            .arg(
                Arg::with_name("services")
                    .help("limit the update to a subset of services")
                    .multiple(true)
                    .required(false),
            )]
    }
}

pub fn update_images(dc: &Dc, trailing: Vec<String>) -> Vec<Task> {
    let pairs = dc.service_img();
    let mut input = {
        if trailing.is_empty() {
            pairs
                .clone()
                .into_iter()
                .map(|i| i.0)
                .collect::<Vec<String>>()
        } else {
            trailing
        }
    };

    {
        input.sort();
        input.dedup();
    }

    let (valid, invalid): (Vec<String>, Vec<String>) = input
        .into_iter()
        .partition(|name| pairs.iter().any(|(service, _img)| *service == *name));

    invalid
        .iter()
        .map(|service| Task::notify_error(missing_service_msg(service.to_string(), &pairs)))
        .chain(
            valid
                .iter()
                .filter_map(|name| pairs.iter().find(|(service, _img)| *service == *name))
                .map(|(_service, image)| format!("docker pull {}", image))
                .map(Task::simple_command),
        )
        .collect()
}

fn missing_service_msg(input: String, services: &[(String, String)]) -> String {
    use ansi_term::Colour::{Cyan, Red};
    format!(
        r#"{}

Did you mean one of these?
{}"#,
        Red.paint(format!(
            "'{}' is not a valid service name in this recipe",
            input
        )),
        Cyan.paint(
            services
                .iter()
                .map(|(service, _)| format!("  {}", service.clone()))
                .collect::<Vec<String>>()
                .join("\n")
        )
    )
}
