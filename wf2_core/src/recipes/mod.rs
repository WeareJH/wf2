use crate::context::Context;
use crate::task::Task;

mod m2_php_7_2;

#[derive(Debug, Clone)]
pub enum Recipe {
    M2 { php: PHP },
}

#[derive(Debug, Clone)]
pub enum PHP {
    //    SevenOne,
    SevenTwo,
}

impl Recipe {
    pub fn resolve(&self, context: &Context) -> Vec<Task> {
        match self {
            Recipe::M2 { php: PHP::SevenTwo } => m2_php_7_2::tasks(&context),
        }
    }
    pub fn m2_php_7_2() -> Recipe {
        Recipe::M2 { php: PHP::SevenTwo }
    }
}
