use crate::recipes::m2::subcommands::composer::{composer, ComposerPassThru};
use crate::recipes::m2::subcommands::dc::{dc_passthru, DcPassThru};
use crate::recipes::m2::subcommands::m::{mage, MPassThru};
use crate::recipes::m2::subcommands::n98::{n98, N98PassThru};
use crate::recipes::m2::subcommands::node::{node, NodePassThru};
use crate::recipes::m2::M2Recipe;
use crate::{context::Context, task::Task};
use std::{cmp, fmt};

///
/// Enum to represent all of the possible pass-thru commands available
///
#[derive(Debug, Clone)]
pub enum M2PassThru {
    Composer,
    Dc,
    Node,
    M,
    N98
}

impl M2PassThru {
    ///
    /// Passthru command names
    ///
    const COMPOSER: &'static str = "composer";
    const DC: &'static str = "dc";
    const NODE: &'static str = "node";
    const MAGE: &'static str = "m";
    const N98_MAGERUN: &'static str = "n98";

    ///
    /// Helper method for converting an enum member to a String
    ///
    pub fn name(&self) -> String {
        match self {
            M2PassThru::Composer => M2PassThru::COMPOSER,
            M2PassThru::Dc => M2PassThru::DC,
            M2PassThru::Node => M2PassThru::NODE,
            M2PassThru::M => M2PassThru::MAGE,
            M2PassThru::N98 => M2PassThru::N98_MAGERUN
        }
        .to_string()
    }
    pub fn resolve_cmd(ctx: &Context, cmd: String, trailing: Vec<String>) -> Option<Vec<Task>> {
        match M2Recipe::dc_tasks(&ctx) {
            Ok(dc) => match cmd {
                ref x if *x == M2PassThru::Dc => Some(dc_passthru(trailing, dc)),
                ref x if *x == M2PassThru::Node => Some(node(trailing, dc)),
                ref x if *x == M2PassThru::Composer => Some(composer(&ctx, trailing)),
                ref x if *x == M2PassThru::M => Some(mage(&ctx, trailing)),
                ref x if *x == M2PassThru::N98 => Some(n98(&ctx, trailing)),
                _ => None,
            },
            Err(e) => Some(Task::task_err_vec(e)),
        }
    }
}

pub fn commands() -> Vec<(String, String)> {
    vec![
        (M2PassThru::Composer, ComposerPassThru::ABOUT),
        (M2PassThru::Dc, DcPassThru::ABOUT),
        (M2PassThru::Node, NodePassThru::ABOUT),
        (M2PassThru::M, MPassThru::ABOUT),
        (M2PassThru::N98, N98PassThru::ABOUT),
    ]
    .into_iter()
    .map(|(name, help)| (name.into(), help.into()))
    .collect()
}

///
/// This allows an instance of M2PassThru to be
/// converted to a string via .into().
///
/// ```
/// use wf2_core::recipes::m2::pass_thru::M2PassThru;
///
/// let single_name: String = M2PassThru::Composer.into();
/// assert_eq!(single_name, "composer");
/// ```
///
impl From<M2PassThru> for String {
    fn from(m2p: M2PassThru) -> Self {
        m2p.name()
    }
}

///
/// Allow the command to be formatted
///
/// ```
/// use wf2_core::recipes::m2::pass_thru::M2PassThru;
/// let expected = "--composer--".to_string();
/// let actual = format!("--{}--", M2PassThru::Composer);
/// assert_eq!(expected, actual);
/// ```
///
impl fmt::Display for M2PassThru {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.name())
    }
}

///
/// Allow a comparison to String
///
/// ```
/// use wf2_core::recipes::m2::pass_thru::M2PassThru;
/// assert_eq!(true, M2PassThru::Composer == String::from("composer"));
/// ```
///
impl cmp::PartialEq<String> for M2PassThru {
    fn eq(&self, other: &String) -> bool {
        self.name() == *other
    }
}

impl cmp::PartialEq<M2PassThru> for String {
    fn eq(&self, other: &M2PassThru) -> bool {
        *self == other.name()
    }
}

///
/// Allow a comparison to &str
///
/// ```
/// use wf2_core::recipes::m2::pass_thru::M2PassThru;
/// assert_eq!(true, M2PassThru::Composer == "composer");
/// ```
///
impl cmp::PartialEq<&str> for M2PassThru {
    fn eq(&self, other: &&str) -> bool {
        &self.name() == other
    }
}
