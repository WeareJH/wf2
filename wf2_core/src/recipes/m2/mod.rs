use crate::context::{Cmd, Context};
use crate::recipes::{m2, Recipe};
use crate::task::Task;

pub mod composer;
pub mod db_dump;
pub mod db_import;
pub mod down;
pub mod eject;
pub mod exec;
pub mod mage;
pub mod npm;
pub mod pull;
pub mod stop;
pub mod up;

pub struct M2Recipe;

impl Recipe for M2Recipe {
    fn resolve_cmd(&self, ctx: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        match cmd {
            Cmd::Up => Some(up::exec(&ctx)),
            Cmd::Down => Some(down::exec(&ctx)),
            Cmd::Stop => Some(stop::exec(&ctx)),
            Cmd::Eject => Some(eject::exec(&ctx)),
            Cmd::Exec { trailing, user } => Some(exec::exec(&ctx, trailing.clone(), user.clone())),
            Cmd::Npm { trailing, .. } => Some(npm::exec(&ctx, trailing.clone())),
            Cmd::Mage { trailing } => Some(mage::exec(&ctx, trailing.clone())),
            Cmd::DBImport { path } => Some(db_import::exec(&ctx, path.clone())),
            Cmd::DBDump => Some(db_dump::exec(&ctx)),
            Cmd::Pull { trailing } => Some(pull::exec(&ctx, trailing.clone())),
        }
    }
}
