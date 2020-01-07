use crate::recipes::m2::services::php::PhpService;
use crate::recipes::m2::services::M2_ROOT;
use crate::recipes::m2::M2Recipe;
use crate::{context::Context, dc_tasks::DcTasks, task::Task, util::path_buf_to_string};
use std::{cmp, fmt, path::PathBuf};

///
/// Enum to represent all of the possible pass-thru commands available
///
#[derive(Debug, Clone)]
pub enum M2PassThru {
    Composer,
    Npm,
    Dc,
    Node,
    M,
}

impl M2PassThru {
    ///
    /// Passthru command names
    ///
    const COMPOSER: &'static str = "composer";
    const NPM: &'static str = "npm";
    const DC: &'static str = "dc";
    const NODE: &'static str = "node";
    const MAGE: &'static str = "m";

    ///
    /// Helper method for converting an enum member to a String
    ///
    pub fn name(&self) -> String {
        match self {
            M2PassThru::Composer => M2PassThru::COMPOSER,
            M2PassThru::Npm => M2PassThru::NPM,
            M2PassThru::Dc => M2PassThru::DC,
            M2PassThru::Node => M2PassThru::NODE,
            M2PassThru::M => M2PassThru::MAGE,
        }
        .to_string()
    }
    pub fn resolve_cmd(ctx: &Context, cmd: String, trailing: Vec<String>) -> Option<Vec<Task>> {
        match M2Recipe::dc_tasks(&ctx) {
            Ok(dc) => match cmd {
                ref x if *x == M2PassThru::Dc => Some(dc_passthru(&ctx, trailing, dc)),
                ref x if *x == M2PassThru::Npm => Some(npm(&ctx, trailing, dc)),
                ref x if *x == M2PassThru::Node => Some(node(&ctx, trailing, dc)),
                ref x if *x == M2PassThru::Composer => Some(composer(&ctx, trailing)),
                ref x if *x == M2PassThru::M => Some(mage(&ctx, trailing)),
                _ => None,
            },
            Err(e) => Some(Task::task_err_vec(e)),
        }
    }
}

///
/// A pass-thru command - where everything after `dc` is passed
/// as-is to docker-compose, without verifying any arguments.
///
pub fn dc_passthru(_ctx: &Context, trailing: Vec<String>, dc: DcTasks) -> Vec<Task> {
    let after: Vec<String> = trailing.into_iter().skip(1).collect();
    vec![dc.cmd_task(after)]
}

pub fn node(_ctx: &Context, trailing: Vec<String>, dc: DcTasks) -> Vec<Task> {
    let dc_command = format!(r#"run {}"#, trailing.join(" "));
    vec![dc.cmd_task(vec![dc_command])]
}

pub fn composer(ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
    PhpService::select(&ctx)
        .map(|service| {
            let exec_command = format!(
                r#"docker exec -it -u www-data {container_name} {trailing_args}"#,
                container_name = service.container_name,
                trailing_args = trailing.join(" ")
            );
            vec![Task::simple_command(exec_command)]
        })
        .unwrap_or_else(Task::task_err_vec)
}

pub fn mage(ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
    PhpService::select(&ctx)
        .map(|service| {
            let full_command = format!(
                r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} ./bin/magento {trailing_args}"#,
                width = ctx.term.width,
                height = ctx.term.height,
                container_name = service.container_name,
                trailing_args = trailing
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<String>>()
                    .join(" ")
            );
            vec![Task::simple_command(full_command)]
        })
        .unwrap_or_else(Task::task_err_vec)
}

pub fn npm(ctx: &Context, trailing: Vec<String>, dc: DcTasks) -> Vec<Task> {
    let dc_command = format!(
        r#"run --workdir {work_dir} {service} {trailing_args}"#,
        work_dir = path_buf_to_string(&PathBuf::from(M2_ROOT).join(ctx.npm_path.clone())),
        service = "node",
        trailing_args = trailing.join(" ")
    );
    vec![dc.cmd_task(vec![dc_command])]
}

pub fn commands() -> Vec<(String, String)> {
    vec![
        (
            M2PassThru::Composer,
            "[m2] Run composer commands with the correct user",
        ),
        (M2PassThru::Npm, "[m2] Run npm commands"),
        (M2PassThru::Dc, "[m2] Run docker-compose commands"),
        (M2PassThru::Node, "[m2] Run commands in the node container"),
        (
            M2PassThru::M,
            "[m2] Execute ./bin/magento commands inside the PHP container",
        ),
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
