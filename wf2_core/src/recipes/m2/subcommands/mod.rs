//!
//! All of the Magento 2 specific sub-commands
//!
use crate::commands::CliCommand;
use db_dump::M2DbDump;
use db_import::M2DbImport;
use doctor::M2Doctor;
use down::M2Down;
use eject::M2Eject;
use exec::M2Exec;
use list_images::M2ListImages;
use m2_playground_cmd::M2PlaygroundCmd;
use pull::M2Pull;
use push::M2Push;
use sql::SqlCmd;
use stop::M2Stop;
use up::M2Up;
use update_images::M2UpdateImages;
use varnish::VarnishCmd;
use xdebug::XdebugCmd;

pub mod composer;
pub mod db_dump;
pub mod db_import;
pub mod dc;
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

pub fn m2_recipe_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![
        Box::new(M2Up),
        Box::new(M2Down),
        Box::new(M2Stop),
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
    ]
}

pub fn m2_recipe_global_subcommands<'a, 'b>() -> Vec<Box<dyn CliCommand<'a, 'b>>> {
    vec![Box::new(M2PlaygroundCmd)]
}
