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

impl Recipe {
    pub fn resolve(&self, context: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        match self {
            Recipe::M2 { php } => match cmd {
                Cmd::Up => Some(m2::up(&context, php)),
                Cmd::Down => Some(m2::down(&context, php)),
                Cmd::Stop => Some(m2::stop(&context, php)),
                Cmd::Exec { trailing } => Some(m2::exec(&context, trailing.clone())),
                Cmd::Mage { trailing } => Some(m2::mage(&context, trailing.clone())),
                _ => None,
            },
        }
    }
}
