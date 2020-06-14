//!
//! pass-thru for `pm2`
//!
//! `pm2` will simply forward all arguments to the `pm2` package - which
//! means ALL valid pm2 commands are valid to run with `wf2`
//!
//! # Example: run the pm2 monitor
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # use wf2_core::recipes::recipe_kinds::RecipeKinds;
//! # let cmd = r#"
//! wf2 pm2 monit
//! # "#;
//! # let commands = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/pwa.yml")
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .commands();
//! # let expected = "docker exec -it wf2__shane__pwa pm2 monit";
//! # assert_eq!(commands, vec![expected]);
//! ```
use crate::context::Context;
use crate::services::pwa::PwaService;
use crate::services::Service;
use crate::task::Task;

pub struct Pm2PassThru;

impl Pm2PassThru {
    pub const NAME: &'static str = "pm2";
    pub const ABOUT: &'static str = "Run PM2 commands";
}

pub fn pm2_pass_thru(ctx: &Context, trailing: &[String]) -> Vec<Task> {
    let container_name = ctx.prefixed_name(PwaService::NAME);
    let exec_command = format!(
        r#"docker exec -it {container_name} {trailing_args}"#,
        container_name = container_name,
        trailing_args = trailing.join(" ")
    );
    vec![Task::simple_command(exec_command)]
}
