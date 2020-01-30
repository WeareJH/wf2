//!
//! Enable and disable Varnish
//!
//! Varnish is not 'enabled' by default - the service starts everytime
//! which is why you may of seen it when running `docker ps`, but it's running
//! in pass-through mode, meaning it will never cache anything.
//!
//! You need to `enable` it manually
//!
//! # Example: enable varnish
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 varnish enable
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml exec varnish varnishadm vcl.use boot0";
//! # assert_eq!(commands, vec![expected])
//! ```
//!
//! # Example: disable varnish
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 varnish disable
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_recipe(RecipeKinds::M2_NAME)
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker-compose -f /users/shane/.wf2_m2_shane/docker-compose.yml exec varnish varnishadm vcl.use boot";
//! # assert_eq!(commands, vec![expected])
//! ```
//! ## Further reading
//!
//! See the [Varnish Service](../../services/varnish/index.html) for more information about
//! enabling Varnish within Magento, debugging tricks and more.
//!
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::varnish::VarnishService;
use crate::recipes::m2::services::M2Service;
use crate::scripts::service_cmd::ServiceCmd;
use crate::task::Task;
use clap::{App, ArgMatches, SubCommand};

#[doc_link::doc_link("/recipes/m2/subcommands/varnish")]
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
            .after_help(VarnishCmd::DOC_LINK)
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
