use crate::condition::{Answer, Con, ConditionFuture};
use core::fmt;
use futures::future::lazy;
use std::io;

pub struct Question {
    pub question: String,
}

impl Question {
    pub fn new(q: impl Into<String>) -> Question {
        Question { question: q.into() }
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "Question: {}", self.question)
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
