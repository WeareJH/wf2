//!
//! Execute commands in the main container.
//!
use crate::commands::CliCommand;
use crate::context::Context;
use crate::recipes::m2::services::php::PhpService;
use crate::task::Task;
use clap::{App, ArgMatches};
use structopt::StructOpt;
use crate::services::elastic_search::ElasticSearchService;
use crate::services::Service;

#[doc_link::doc_link("/recipes/m2/subcommands/exec")]
pub struct M2Install;

impl M2Install {
    const NAME: &'static str = "exec";
    const ABOUT: &'static str = "Execute commands in the main container";
}

impl<'a, 'b> CliCommand<'a, 'b> for M2Install {
    fn name(&self) -> String {
        String::from(M2Install::NAME)
    }
    fn exec(&self, _matches: Option<&ArgMatches>, ctx: &Context) -> Option<Vec<Task>> {
        Some(exec(ctx))
    }
    fn subcommands(&self, _ctx: &Context) -> Vec<App<'a, 'b>> {
        vec![
            App::new(M2Install::NAME)
            .about(M2Install::ABOUT)
            .after_help(M2Install::DOC_LINK)
        ]
    }
}


///
/// Alias for `docker exec` inside the PHP Container.
///
/// Note: if the command you're running requires flags like `-h`, then you
/// need to place `--` directly after `exec` (see below)
///
pub fn exec(ctx: &Context) -> Vec<Task> {
    PhpService::select(&ctx).map(|service| {
        let mut args = vec![
            "bin/magento",
            "setup:install",
            "--db-host=db",
            "--db-name=$MYSQL_DATABASE",
            "--db-user=$MYSQL_USER",
            "--db-password=$MYSQL_PASSWORD",
            "--base-url=$MAGE_HOST",
            "--base-url-secure=$MAGE_HOST",
            "--admin-firstname="$MAGE_ADMIN_FIRSTNAME"",
            "--admin-lastname="$MAGE_ADMIN_LASTNAME"",
            "--admin-email=$MAGE_ADMIN_EMAIL",
            "--admin-user=$MAGE_ADMIN_USER",
            "--admin-password=$MAGE_ADMIN_PASS",
            "--backend-frontname=$MAGE_BACKEND_FRONTNAME",
            "--use-secure=1",
            "--use-secure-admin=1",
            "--cleanup-database -vvv",
        ].join("\\");

        let end = "|| { exit 1; }";
        if requires_es {
            args.push(&format!("--elasticsearch-host={}", ElasticSearchService::NAME))
        }
        let exec_command = format!(
            r#"docker exec -it -u {user} -e COLUMNS="{width}" -e LINES="{height}" {container_name} {trailing_args}"#,
            user = user,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = service.container_name,
            trailing_args = str
        );
        vec![Task::simple_command(exec_command)]
    }).unwrap_or_else(Task::task_err_vec)
}
