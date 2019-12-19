use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::m2_vars::NGINX_UPSTREAM_OUTPUT_FILE;
use crate::recipes::m2::services::M2Services;
use crate::recipes::m2::M2Recipe;

use crate::task::Task;
use clap::{App, ArgMatches, SubCommand};

pub struct XdebugCmd(String);

const NAME: &'static str = "xdebug";

impl XdebugCmd {
    const ENABLE: &'static str = "enable";
    const DISABLE: &'static str = "disable";

    pub fn new() -> XdebugCmd {
        XdebugCmd(String::from(NAME))
    }
}

impl<'a, 'b> CliCommand<'a, 'b> for XdebugCmd {
    fn name(&self) -> String {
        String::from(NAME)
    }

    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Vec<Task> {
        let enabled = matches.map(|m| m.subcommand_name()).and_then(|n| match n {
            Some(XdebugCmd::ENABLE) => Some(true),
            Some(XdebugCmd::DISABLE) => Some(false),
            _ => None,
        });

        if enabled.is_none() {
            return vec![Task::notify_error("missing `enable` or `disable`")];
        }

        let enabled = enabled.expect("guarded");
        let main = if enabled {
            M2Services::PHP_DEBUG
        } else {
            M2Services::PHP
        };
        let string = nginx_upstream(main, M2Services::PHP_DEBUG);
        let msg = if enabled {
            "XDebug Enabled"
        } else {
            "XDebug Disabled"
        };
        let dc = M2Recipe::dc_tasks(ctx);

        if let Err(_e) = dc {
            return vec![Task::notify_error(
                "couldn't create the docker-compose task",
            )];
        }

        let dc = dc.expect("guarded above");

        vec![
            Task::notify_info("updating upstream.conf"),
            write_file(ctx, string),
            Task::notify_info("reloading nginx conf"),
            dc.cmd_task(vec!["exec", "nginx", "nginx", "-s", "reload"]),
            Task::notify_info(msg),
        ]
    }

    fn subcommands(&self) -> Vec<App<'a, 'b>> {
        vec![App::new(NAME)
            .about("Enable or disable XDebug for M2")
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

fn write_file(ctx: &Context, content: String) -> Task {
    Task::file_write(
        ctx.file_prefix.join(NGINX_UPSTREAM_OUTPUT_FILE),
        "write the upstream file",
        content,
    )
}

pub fn nginx_upstream<S>(backend: S, backend_debug: S) -> String
where
    S: Into<String>,
{
    format!(
        r#"upstream fastcgi_backend {{
  server {}:9000;
}}

upstream fastcgi_backend_debug {{
  server {}:9000;
}}
    "#,
        backend.into(),
        backend_debug.into()
    )
}
