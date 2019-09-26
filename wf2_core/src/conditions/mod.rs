use crate::condition::{Answer, Con, ConditionFuture};
use crate::task::Task::File;
use futures::{future::lazy, future::Future, IntoFuture};
use std::io::Error;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct Question {
    pub question: String,
}

impl Question {
    pub fn new(q: impl Into<String>) -> Question {
        Question { question: q.into() }
    }
}

impl Con for Question {
    fn exec(&self) -> ConditionFuture {
        let q = self.question.clone();
        Box::new(lazy(move || loop {
            println!("{} Y/n", q);
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .ok()
                .expect("Couldn't read line");

            match input.trim() {
                "y" | "Y" => return Ok(Answer::Yes),
                "n" | "N" => return Ok(Answer::No),
                _ => {
                    println!("Sorry, we didn't recognise that answer, try again");
                    continue;
                }
            };
        }))
    }
}

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

pub struct FilePresent {
    pub path: PathBuf,
}
impl FilePresent {
    pub fn new(p: impl Into<PathBuf>) -> FilePresent {
        FilePresent { path: p.into() }
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
