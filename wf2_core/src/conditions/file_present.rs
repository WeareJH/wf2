use crate::condition::{Answer, Con, ConditionFuture};
use core::fmt;
use futures::future::lazy;
use std::path::{Path, PathBuf};

pub struct FilePresent {
    pub path: PathBuf,
}
impl FilePresent {
    pub fn new(p: impl Into<PathBuf>) -> FilePresent {
        FilePresent { path: p.into() }
    }
}

impl fmt::Display for FilePresent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "FilePresent: {}", self.path.display())
    }
}

impl Con for FilePresent {
    fn exec(&self) -> ConditionFuture {
        let path = self.path.clone();
        Box::new(lazy(move || {
            if Path::exists(path.as_path()) {
                Ok(Answer::Yes)
            } else {
                Ok(Answer::No)
            }
        }))
    }
}
