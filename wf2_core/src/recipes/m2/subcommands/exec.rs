//!
//! Execute commands in the main container.
//!
//! Almost every time you `sh` into a container to run commands, you
//! should probably use `exec` instead.
//!
//! ## List files in the main container
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 exec ls
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/config_01.yaml")
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .file_ops_commands();
//! # assert_eq!(commands, vec!["docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__shane__php ls"])
//! ```
//!
//! ## Run as a command as `root`
//!
//! Be careful with this, but if you *absolutely* need to run a command as root, place the '-r'
//! flag before the cammand
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 exec -r some-command
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/config_01.yaml")
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .file_ops_commands();
//! # assert_eq!(commands, vec!["docker exec -it -u root -e COLUMNS=\"80\" -e LINES=\"30\" wf2__shane__php some-command"])
//! ```
//!
//! ## Run a command that has flags
//!
//! A limitiation of `exec` is that it doesn't know when it's *own* arguments are finished, and
//! when your command begins. This is not a problem for commands without flags, but if you need
//! flags, just use `--` to separate `exec` from your command
//!
//! ```
//! # use wf2_core::test::Test;
//! # use wf2_core::cli::cli_input::CLIInput;
//! # let cmd = r#"
//! wf2 exec -- rm -rf pub/static
//! # "#;
//! # let (commands, ..) = Test::from_cmd(cmd)
//! #     .with_file("../fixtures/config_01.yaml")
//! #     .with_cli_input(CLIInput::from_cwd("/users/shane"))
//! #     .file_ops_commands();
//! # assert_eq!(commands, vec!["docker exec -it -u www-data -e COLUMNS=\"80\" -e LINES=\"30\" wf2__shane__php rm -rf pub/static"])
//! ```
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::php::PhpService;
use crate::task::Task;
use clap::{App, ArgMatches};
use structopt::StructOpt;

#[doc_link::doc_link("/recipes/m2/subcommands/exec")]
pub struct M2Exec;

impl M2Exec {
    const NAME: &'static str = "exec";
    const ABOUT: &'static str = "[m2] Execute commands in the main container";
}

#[derive(StructOpt)]
struct Opts {
    #[structopt(short, long)]
    root: bool,
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Exec {
    fn name(&self) -> String {
        String::from(M2Exec::NAME)
    }
    fn exec(&self, matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        let matches = matches.expect("guarded by Clap");
        let opts: Opts = Opts::from_clap(&matches);
        let user = if opts.root { "root" } else { "www-data" };
        let trailing = get_trailing(matches);
        Some(exec(ctx, trailing, user))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![App::new(M2Exec::NAME)
            .about(M2Exec::ABOUT)
            .after_help(M2Exec::DOC_LINK)
            .args_from_usage(
                "-r --root 'Execute commands as root'
                                  [cmd]... 'Trailing args'",
            )]
    }
}

//
// Extract sub-command trailing arguments, eg:
//
//                  captured
//             |-----------------|
//    wf2 exec  ./bin/magento c:f
//
fn get_trailing(sub_matches: &ArgMatches) -> Vec<String> {
    let output = match sub_matches.values_of("cmd") {
        Some(cmd) => cmd.collect::<Vec<&str>>(),
        None => vec![],
    };
    output.into_iter().map(|x| x.to_string()).collect()
}

///
/// Alias for `docker exec` inside the PHP Container.
///
/// Note: if the command you're running requires flags like `-h`, then you
/// need to place `--` directly after `exec` (see below)
///
pub fn exec(ctx: &Context, trailing: Vec<String>, user: &str) -> Vec<Task> {
    PhpService::select(&ctx).map(|service| {
        let exec_command = format!(
            r#"docker exec -it -u {user} -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
            user = user,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = service.container_name,
            trailing_args = trailing.join(" ")
        );
        vec![Task::simple_command(exec_command)]
    }).unwrap_or_else(Task::task_err_vec)
}
