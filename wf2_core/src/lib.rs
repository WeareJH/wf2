//!
//! # wf2 - reliable docker environments
//!
//! wf2 is a CLI tool for creating reliable, opinionated docker
//! environments for local development.
//! # wf2 ![](https://github.com/WeareJH/wf2/workflows/.github/workflows/test.yml/badge.svg)
//!
//! ## Express Install
//!
//! Simply run the following in your terminal
//!
//! ```shell
//! zsh <(curl -L https://raw.githubusercontent.com/WeareJH/wf2/master/express-install.sh) && source ~/.zshrc
//! ```
//!
//! ## Install
//! `wf2` is distributed as a single binary with everything packaged inside -
//! this means you *do not* need PHP or Composer installed on your machine.
//!
//! 1. Download the latest version from the [releases page](https://github.com/WeareJH/wf2/releases)
//! 2. Make the file executable: (assuming it downloaded to the `Downloads` folder)
//!
//!     `chmod +x ~/Downloads/wf2`
//!
//! 3. Move the executable from your Downloads folder to /opt
//!
//!     `sudo mv ~/Downloads/wf2 /opt`
//!
//!     - If "opt" does not exist run the command below
//!
//!         `sudo mkdir /opt`
//!
//!     - Then make sure the permissions are correct on the folder
//!
//!         `sudo chown -R $(whoami) /opt`
//!
//! 5. Add this to the bottom of your *zshrc* or *bash_profile*:
//!
//!     `export PATH="$PATH:/opt"`
//!
//! 6. Use the following command to refresh any already open terminals
//!
//!     `source ~/.zshrc`
//!
//! 7. Or for bash users
//!
//!     `source ~/.bash_profile`
//!
//! 8. Type the following command to check all is installed OK:
//!
//!     `wf2`
//!
//! 9. You should see the same output as below (in features):
//!
//! ## Help
//!
//! For help on the commands available, run the following:
//!
//! ```shell script
//! wf2 --help
//! ```
//!
//! `--help` is recipe specific. So if you're in a M2 project, you'll only see
//! commands that are relevant to M2 sites.
//!
//! If you just want to explore what the the wf2 tool can do in each recipe, just use
//! the `--recipe` command
//!
//! ```shell script
//! # See M2 help
//! wf2 --recipe M2 --help
//!
//! # See Wp help
//! wf2 --recipe Wp --help
//! ```
//!
//! ## Contributing.
//!
//! Before pushing any code, run the following to ensure you catch
//! problems before they get to CI
//!
//! ```shell script
//! bash pre-push.sh
//! ```
//!
#![allow(
    clippy::trivially_copy_pass_by_ref,
    clippy::float_cmp,
    clippy::needless_doctest_main,
    clippy::module_inception
)]

#[macro_use]
extern crate clap;

#[macro_use]
extern crate prettytable;

#[macro_use]
extern crate serde_derive;
extern crate serde;

#[macro_use]
extern crate from_file_derive;

#[macro_use]
extern crate failure;

#[doc(hidden)]
pub mod cli;
#[doc(hidden)]
pub mod cmd;
pub mod commands;
#[doc(hidden)]
pub mod condition;
#[doc(hidden)]
pub mod conditions;
#[doc(hidden)]
pub mod context;
#[doc(hidden)]
pub mod date_util;
#[doc(hidden)]
pub mod dc;
#[doc(hidden)]
pub mod dc_service;
#[doc(hidden)]
pub mod dc_tasks;
#[doc(hidden)]
pub mod dc_volume;
#[doc(hidden)]
pub mod file;
#[doc(hidden)]
pub mod file_op;
#[doc(hidden)]
pub mod output;
#[doc(hidden)]
pub mod php;
pub mod recipes;
#[doc(hidden)]
pub mod scripts;
#[doc(hidden)]
pub mod task;
#[doc(hidden)]
pub mod tasks;
#[doc(hidden)]
pub mod test;
#[doc(hidden)]
pub mod util;
#[doc(hidden)]
pub mod vars;
#[doc(hidden)]
pub mod zip_utils;

use futures::{future::lazy, future::Future, stream::iter_ok, Stream};

use crate::{
    task::TaskError,
    task::{as_future, Task},
};

use crate::condition::{Answer, Con, ConditionFuture};

#[doc(hidden)]
pub struct WF2;

#[doc(hidden)]
pub type SeqFuture = Box<dyn Future<Item = (), Error = (usize, TaskError)> + Send>;
#[doc(hidden)]
pub type CondFut = Box<dyn Future<Item = (), Error = (usize, TaskError)> + Send>;

#[doc(hidden)]
#[derive(Debug)]
pub enum Reason {
    Bail,
    Error { e: String },
}

impl WF2 {
    ///
    /// Create a future that will execute all of the tasks in sequence
    ///
    pub fn sequence(tasks: Vec<Task>) -> SeqFuture {
        Box::new(lazy(move || {
            // convert the list of tasks into a sequence
            let as_futures = tasks
                .into_iter()
                .enumerate()
                .map(|(index, task)| as_future(task, index));

            // iterate through every task and execute
            iter_ok(as_futures).for_each(move |f| {
                f.then(move |out| {
                    match out {
                        // Task was successful, can continue
                        Ok(..) => Ok(()),
                        // Task error, halt execution
                        Err(te) => Err((te.index, te)),
                    }
                })
            })
        }))
    }

    pub fn conditions(conditions: Vec<Box<dyn Con>>) -> ConditionFuture {
        Box::new(lazy(move || {
            let as_futures = conditions.into_iter().map(|c| {
                c.exec().then(|a| match a {
                    Ok(Answer::Yes) => Ok(Answer::Yes),
                    Ok(Answer::No) => Err(Reason::Bail),
                    Err(e) => Err(Reason::Error { e }),
                })
            });
            futures::collect(as_futures).then(|res| match res {
                Ok(answers) => {
                    if Answer::all_yes(answers) {
                        Ok(Answer::Yes)
                    } else {
                        Ok(Answer::No)
                    }
                }
                Err(reason) => match reason {
                    Reason::Bail => Ok(Answer::No),
                    Reason::Error { e } => Err(e),
                },
            })
        }))
    }
}
