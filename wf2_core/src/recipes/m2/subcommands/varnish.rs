use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::varnish::VarnishService;
use crate::recipes::m2::services::M2Service;
use crate::scripts::service_cmd::ServiceCmd;
use crate::task::Task;
use clap::{App, ArgMatches, SubCommand};

pub struct VarnishCmd;

impl VarnishCmd {
    const NAME: &'static str = "varnish";
    const ABOUT: &'static str = "[m2] Enable or disable Varnish for M2";

    const ENABLE: &'static str = "enable";
    const DISABLE: &'static str = "disable";

    const ENABLE_CMD: &'static str = "varnishadm vcl.use boot0";
    const DISABLE_CMD: &'static str = "varnishadm vcl.use boot";
}

impl<'a, 'b> CliCommand<'a, 'b> for VarnishCmd {
    fn name(&self) -> String {
        String::from(VarnishCmd::NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        matches
            .map(|m| m.subcommand_name())
            .and_then(|n| match n {
                Some(VarnishCmd::ENABLE) => Some(VarnishCmd::ENABLE_CMD),
                Some(VarnishCmd::DISABLE) => Some(VarnishCmd::DISABLE_CMD),
                _ => None,
            })
            .map(|cmd| -> Task {
                let as_cmd = ServiceCmd::running_cmd(VarnishService::NAME, cmd, ctx);
                as_cmd.into()
            })
            .map_or(
                Some(vec![Task::notify_error("missing `enable` or `disable`")]),
                |t| Some(vec![t]),
            )
    }

    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(VarnishCmd::NAME)
            .about(VarnishCmd::ABOUT)
            .after_help("Enable: `wf2 varnish enable`. Disable: `wf2 varnish disable`.")
            .subcommands(vec![
                SubCommand::with_name(VarnishCmd::ENABLE)
                    .display_order(0)
                    .about("Enable Varnish"),
                SubCommand::with_name(VarnishCmd::DISABLE)
                    .display_order(0)
                    .about("Disable Varnish"),
            ])]
    }
}
