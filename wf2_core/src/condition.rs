use futures::{future::lazy, future::Future, IntoFuture};
use std::fmt;
use std::fmt::Debug;

///
/// A trait to represent a thread-safe condition
///
pub trait Con: Send + Sync {
    fn exec(&self) -> ConditionFuture;
}

impl Debug for dyn Con {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum Answer {
    Yes,
    No,
}

impl Answer {
    pub fn is_yes(&self) -> bool {
        match self {
            Answer::Yes => true,
            Answer::No => false,
        }
    }
    pub fn all_yes(answers: Vec<Answer>) -> bool {
        answers.iter().all(|a| a.is_yes())
    }
}

pub type ConditionFuture = Box<dyn Future<Item = Answer, Error = String> + Send>;
