use crate::{
    context::{Cmd, Context},
    task::Task,
};

pub mod m2;
pub mod magento_2;

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
                Cmd::Up => Some(m2::up::exec(&context, php)),
                Cmd::Down => Some(m2::down::exec(&context, php)),
                Cmd::Stop => Some(m2::stop::exec(&context, php)),
                Cmd::Eject => Some(m2::eject::exec(&context, php)),
                Cmd::Exec { trailing, user } => {
                    Some(m2::exec::exec(&context, trailing.clone(), user.clone()))
                }
                Cmd::Mage { trailing } => Some(m2::mage::exec(&context, trailing.clone())),
                Cmd::DBImport { path } => Some(m2::db_import::exec(&context, path.clone())),
                Cmd::DBDump => Some(m2::db_dump::exec(&context)),
                Cmd::Pull { trailing } => Some(m2::pull::exec(&context, trailing.clone())),
            },
        }
    }
}

// wf2 pull vendor
// docker cp wf2__php:/var/www/vendor
