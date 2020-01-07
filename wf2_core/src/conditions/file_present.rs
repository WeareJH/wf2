use crate::condition::{Answer, Con, ConditionFuture};
use crate::output::output;
use core::fmt;
use futures::future::lazy;
use std::path::{Path, PathBuf};

pub struct FilePresent {
    pub path: PathBuf,
    pub invert: bool,
}

impl FilePresent {
    pub fn new(p: impl Into<PathBuf>, invert: bool) -> FilePresent {
        FilePresent {
            path: p.into(),
            invert,
        }
    }
}

impl fmt::Display for FilePresent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let string = output(
            format!(
                "File {}present check",
                if self.invert { "NOT " } else { "" }
            ),
            self.path.display().to_string(),
        );
        write!(f, "{}", string)
    }
}

impl Con for FilePresent {
    fn exec(&self) -> ConditionFuture {
        let path = self.path.clone();
        let invert = self.invert;
        Box::new(lazy(move || {
            let exists = Path::exists(path.as_path());
            if invert {
                if exists {
                    Ok(Answer::No)
                } else {
                    Ok(Answer::Yes)
                }
            } else if exists {
                Ok(Answer::Yes)
            } else {
                Ok(Answer::No)
            }
        }))
    }
}
