//!
//! All of the Magento 2 specific sub-commands
//!
use crate::commands::{CliCommand, Commands};
use db_dump::M2DbDump;
use db_import::M2DbImport;
use doctor::M2Doctor;

use eject::M2Eject;
use exec::M2Exec;
use list_images::M2ListImages;
use m2_playground_cmd::M2PlaygroundCmd;
use pull::M2Pull;
use push::M2Push;
use sql::SqlCmd;

use crate::context::Context;
use crate::recipes::m2::M2Recipe;
use crate::subcommands::down::DcDown;
use crate::subcommands::stop::DcStop;
use up::M2Up;
use update_images::M2UpdateImages;
use varnish::VarnishCmd;
use xdebug::XdebugCmd;
use crate::recipes::m2::subcommands::install::M2Install;

pub mod composer;
pub mod db_dump;
pub mod db_import;
pub mod doctor;
pub mod down;
pub mod eject;
pub mod exec;
pub mod list_images;
pub mod m;
pub mod m2_playground;
#[doc(hidden)]
pub mod m2_playground_cmd;
#[doc(hidden)]
pub mod m2_playground_help;
pub mod n98;
pub mod node;
pub mod pull;
pub mod push;
pub mod sql;
pub mod stop;
pub mod up;
#[doc(hidden)]
pub mod up_help;
pub mod update_images;
pub mod varnish;
pub mod xdebug;
pub mod install;

impl<'a, 'b> Commands<'a, 'b> for M2Recipe {
    fn subcommands(&self, _ctx: &Context) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        vec![
            Box::new(M2Up),
            Box::new(DcDown),
            Box::new(DcStop),
            Box::new(M2DbImport),
            Box::new(M2DbDump),
            Box::new(M2Doctor),
            Box::new(M2Push),
            Box::new(M2Eject),
            Box::new(M2Exec),
            Box::new(M2Pull),
            Box::new(VarnishCmd),
            Box::new(M2ListImages),
            Box::new(M2UpdateImages),
            Box::new(XdebugCmd),
            Box::new(SqlCmd),
            Box::new(M2Install),
        ]
    }
    fn global_subcommands(&self) -> Vec<Box<dyn CliCommand<'a, 'b>>> {
        vec![Box::new(M2PlaygroundCmd)]
    }
}
