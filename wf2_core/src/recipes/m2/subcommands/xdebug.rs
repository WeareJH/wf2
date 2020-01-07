use crate::commands::CliCommand;
use crate::context::Context;

use crate::recipes::m2::M2Recipe;

use crate::file::File;
use crate::task::Task;
use clap::{App, ArgMatches, SubCommand};

use crate::recipes::m2::templates::nginx_upstream::NginxUpstream;

#[derive(Default)]
pub struct XdebugCmd;

impl XdebugCmd {
    const NAME: &'static str = "xdebug";
    const ABOUT: &'static str = "[m2] Enable or disable XDebug for M2";

    const ENABLE: &'static str = "enable";
    const DISABLE: &'static str = "disable";
}

impl<'a, 'b> CliCommand<'a, 'b> for XdebugCmd {
    fn name(&self) -> String {
        String::from(XdebugCmd::NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let enabled = matches.map(|m| m.subcommand_name()).and_then(|n| match n {
            Some(XdebugCmd::ENABLE) => Some(true),
            Some(XdebugCmd::DISABLE) => Some(false),
            _ => None,
        });

        if enabled.is_none() {
            return Some(vec![Task::notify_error("missing `enable` or `disable`")]);
        }

        let enabled = enabled.expect("guarded");

        let msg = if enabled {
            "XDebug Enabled"
        } else {
            "XDebug Disabled"
        };

        let nginx_upstream = NginxUpstream::from_ctx(ctx);

        if let Err(e) = nginx_upstream {
            return Some(Task::task_err_vec(e));
        }

        let mut nginx_upstream = nginx_upstream.expect("guarded");

        let dc = M2Recipe::dc_tasks(ctx);

        if let Err(_e) = dc {
            return Some(vec![Task::notify_error(
                "couldn't create the docker-compose task",
            )]);
        }

        let dc = dc.expect("guarded above");

        Some(vec![
            Task::notify_info("updating upstream.conf"),
            nginx_upstream.toggle_xdebug(enabled).write_task(),
            Task::notify_info("reloading nginx conf"),
            dc.cmd_task(vec!["exec", "nginx", "nginx", "-s", "reload"]),
            Task::notify_info(msg),
        ])
    }

    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(XdebugCmd::NAME)
            .about(XdebugCmd::ABOUT)
            .after_help("Enable: `wf2 xdebug enable`. Disable: `wf2 xdebug disable`.")
            .subcommands(vec![
                SubCommand::with_name(XdebugCmd::ENABLE)
                    .display_order(0)
                    .about("Enable XDebug"),
                SubCommand::with_name(XdebugCmd::DISABLE)
                    .display_order(0)
                    .about("Disable XDebug"),
            ])]
    }
}
