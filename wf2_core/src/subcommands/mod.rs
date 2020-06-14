use crate::cmd::PassThruCmd;
use crate::context::Context;
use crate::task::Task;

pub mod dc;
pub mod down;
pub mod pm2;
pub mod stop;

pub trait PassThru {
    fn resolve(&self, _ctx: &Context, _cmd: &PassThruCmd) -> Option<Vec<Task>> {
        None
    }
    fn names(&self, _ctx: &Context) -> Vec<(String, String)> {
        vec![]
    }
}
