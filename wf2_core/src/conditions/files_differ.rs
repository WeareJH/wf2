use crate::condition::{Answer, Con, ConditionFuture};
use core::fmt;
use futures::future::lazy;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FilesDiffer {
    pub left: PathBuf,
    pub right: PathBuf,
}

impl FilesDiffer {
    pub fn new(left: impl Into<PathBuf>, right: impl Into<PathBuf>) -> FilesDiffer {
        FilesDiffer {
            left: left.into(),
            right: right.into(),
        }
    }
}

impl fmt::Display for FilesDiffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "FilesDiffer: left: {}, right: {}",
            self.left.display(),
            self.right.display()
        )
    }
}

impl Con for FilesDiffer {
    fn exec(&self) -> ConditionFuture {
        let l = self.left.clone();
        let r = self.right.clone();
        Box::new(lazy(move || {
            let l_content = fs::read(Path::new(&l));
            let r_content = fs::read(Path::new(&r));
            match (l_content, r_content) {
                (Ok(l), Ok(r)) => {
                    if l != r {
                        Ok(Answer::Yes)
                    } else {
                        Ok(Answer::No)
                    }
                }
                (Ok(..), Err(e)) => Err(e.to_string()),
                (Err(e), Ok(..)) => Err(e.to_string()),
                (Err(e), Err(..)) => Err(e.to_string()),
            }
        }))
    }
}
