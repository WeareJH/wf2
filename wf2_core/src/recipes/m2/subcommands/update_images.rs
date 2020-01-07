use crate::commands::CliCommand;
use crate::context::Context;
use crate::dc::Dc;
use crate::recipes::m2::M2Recipe;
use crate::task::Task;
use clap::{App, Arg, ArgMatches};
use structopt::StructOpt;

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
