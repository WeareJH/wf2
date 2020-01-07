use crate::commands::CliCommand;
use crate::context::Context;
use crate::task::Task;
use clap::{App, ArgMatches};

use crate::recipes::m2::M2Recipe;
use crate::util::two_col;

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
        vec![App::new(M2ListImages::NAME).about(M2ListImages::ABOUT)]
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
