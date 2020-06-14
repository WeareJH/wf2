//!
//! Enable and disable Xdebug
//!
//! Xdebug is not 'enabled' by default since it causes a big slow down on every request.
//!
//! So, when you need to debug something, you have 2 options:
//!
//! - ## Option 1: Add `?debug=true` to a request.
//!     This is especially useful when working with APIs - since you can leave Xdebug off
//!     which will prevent things like the admin getting really slow - but then have it working
//!     on the the single endpoint you're working on.
//!
//!     For example, if you want to debug a `/graphql` request, there no need to have xdebug
//!     enabled sitewite, you can just change the path to `/graphql?debug=true`
//!
//! - ## Option 2: enable xdebug globally.
//!    You can just 'flip the switch' so to speak, and have xdebug enabled for a short period of
//!    time, but it will trigger for every request.
//!
//!    Image a situation where you just need to debug 1 api request at the end of a checkout.
//!    Since enabling/disabling xdebug does not lose any data/sessions, you can just switch it on
//!    at the very last moment.
//!
//! # Example: enable xdebug
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 xdebug enable
//! # "#;
//! # let _tasks = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .tasks();
//! ```
//!
//! # Example: disable xdebug
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 xdebug disable
//! # "#;
//! # let _tasks = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .tasks();
//! ```
//! ## Further reading
//!
//! See the [Xdebug Service](../../services/xdebug/index.html) for more information.
//!
use crate::commands::CliCommand;
use crate::context::Context;

use crate::file::File;
use crate::task::Task;
use clap::{App, ArgMatches, SubCommand};

use crate::recipes::m2::output_files::nginx_upstream::NginxUpstream;
use crate::recipes::recipe_kinds::RecipeKinds;

#[doc_link::doc_link("/recipes/m2/subcommands/xdebug")]
#[derive(Default)]
pub struct XdebugCmd;

impl XdebugCmd {
    const NAME: &'static str = "xdebug";
    const ABOUT: &'static str = "Enable or disable XDebug for M2";

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

        let recipe = RecipeKinds::select(ctx.recipe.expect("recipe is always resolved here"));
        let dc = recipe.dc_tasks(ctx);

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
            .after_help(XdebugCmd::DOC_LINK)
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
