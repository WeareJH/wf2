use crate::context::{Cmd, Context};
use crate::task::Task;

mod m2;

#[derive(Debug, Clone)]
pub enum Recipe {
    M2 { php: PHP },
}

#[derive(Debug, Clone)]
pub enum PHP {
    SevenOne,
    SevenTwo,
}

///
/// The goal here is to have a single place tie a recipe to a function
/// that can return some tasks.
///
impl Recipe {
    pub fn resolve(&self, context: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        match self {
            Recipe::M2 { php } => match cmd {
                Cmd::Up => Some(m2::up(&context, php)),
                Cmd::Down => Some(m2::down(&context, php)),
                Cmd::Stop => Some(m2::stop(&context, php)),
                Cmd::Eject => Some(m2::eject(&context, php)),
                Cmd::Exec { trailing, user } => Some(m2::exec(&context, trailing.clone(), user.clone())),
                Cmd::Mage { trailing } => Some(m2::mage(&context, trailing.clone())),
                Cmd::DBImport { path } => Some(m2::db_import(&context, path.clone())),
                Cmd::DBDump => Some(m2::db_dump(&context)),
            },
        }
    }
}
