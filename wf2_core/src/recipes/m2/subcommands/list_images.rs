//!
//! List the service names + images used in the current project/recipe
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 list-images
//! # "#;
//! # let _tasks = Test::from_cmd(cmd).with_recipe(RecipeKinds::M2_NAME).tasks();
//! ```
//! The output will be *something* along these lines (this is not a live list, so please
//! run the command locally to see up-to-date information)
//!
//! ```txt
//!    blackfire        blackfire/blackfire
//!    traefik          traefik:1.7
//!    unison           wearejh/unison
//!    varnish          wearejh/magento-varnish:latest
//!    node             wearejh/node:8-m2
//!    db               mysql:5.6
//!    redis            redis:3-alpine
//!    nginx            wearejh/nginx:stable-m2
//!    php-debug        wearejh/php:7.3-m2
//!    rabbitmq         rabbitmq:3.7-management-alpine
//!    elasticsearch    wearejh/elasticsearch:7.6-m2
//!    php              wearejh/php:7.3-m2
//!    mail             mailhog/mailhog
//! ```
use crate::commands::CliCommand;
use crate::context::Context;
use crate::task::Task;
use clap::{App, ArgMatches};

use crate::recipes::m2::M2Recipe;
use crate::util::two_col;

#[doc_link::doc_link("/recipes/m2/subcommands/list_images")]
pub struct M2ListImages;

impl M2ListImages {
    const NAME: &'static str = "list-images";
    const ABOUT: &'static str = "[m2] List the images used in the current recipe";
}

impl<'a, 'b> CliCommand<'a, 'b> for M2ListImages {
    fn name(&self) -> String {
        String::from(M2ListImages::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(list_images(&ctx))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2ListImages::NAME)
            .about(M2ListImages::ABOUT)
            .after_help(M2ListImages::DOC_LINK)]
    }
}

pub fn list_images(ctx: &Context) -> Vec<Task> {
    let dc = M2Recipe::dc(&ctx);
    match dc {
        Ok(dc) => {
            let pairs = dc.service_img();
            vec![Task::notify(two_col(pairs))]
        }
        Err(e) => Task::task_err_vec(e),
    }
}
