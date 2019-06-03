use crate::{
    context::{Cmd, Context},
    recipes::php::PHP,
    task::Task,
};

pub mod m2;
pub mod magento_2;
pub mod php;

#[derive(Debug, Clone)]
pub enum Recipe {
    M2,
}

///
/// The goal here is to have a single place tie a recipe to a function
/// that can return some tasks.
///
impl Recipe {
    pub fn resolve(&self, context: &Context, cmd: Cmd) -> Option<Vec<Task>> {
        match self {
            Recipe::M2 => match cmd {
                Cmd::Up => Some(m2::up::exec(&context)),
                Cmd::Down => Some(m2::down::exec(&context)),
                Cmd::Stop => Some(m2::stop::exec(&context)),
                Cmd::Eject => Some(m2::eject::exec(&context)),
                Cmd::Exec { trailing, user } => {
                    Some(m2::exec::exec(&context, trailing.clone(), user.clone()))
                }
                Cmd::DockerCompose { trailing, .. } => {
                    Some(m2::docker_compose::exec(&context, trailing.clone()))
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
