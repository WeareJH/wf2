use crate::{
    context::Context,
    dc_tasks::DcTasks,
    recipes::m2::{m2_vars::M2Vars, php_container::PhpContainer},
    task::Task,
    util::path_buf_to_string,
};
use std::{cmp, fmt, path::PathBuf};

const COMPOSER: &str = "composer";
const NPM: &str = "npm";
const DC: &str = "dc";
const NODE: &str = "node";
const M: &str = "m";

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
    /// Helper method for converting an enum member to a String
    ///
    pub fn name(&self) -> String {
        match self {
            M2PassThru::Composer => COMPOSER,
            M2PassThru::Npm => NPM,
            M2PassThru::Dc => DC,
            M2PassThru::Node => NODE,
            M2PassThru::M => M,
        }
        .to_string()
    }
    pub fn resolve_cmd(
        ctx: &Context,
        env: &M2Vars,
        cmd: String,
        trailing: Vec<String>,
        dc: DcTasks,
    ) -> Option<Vec<Task>> {
        match cmd {
            ref x if *x == M2PassThru::Dc => Some(M2PassThru::dc(&ctx, &env, trailing, dc)),
            ref x if *x == M2PassThru::Npm => Some(M2PassThru::npm(&ctx, &env, trailing, dc)),
            ref x if *x == M2PassThru::Node => Some(M2PassThru::node(&ctx, &env, trailing, dc)),
            ref x if *x == M2PassThru::Composer => Some(M2PassThru::composer(&ctx, trailing)),
            ref x if *x == M2PassThru::M => Some(M2PassThru::mage(&ctx, trailing)),
            _ => None,
        }
    }

    ///
    /// A pass-thru command - where everything after `dc` is passed
    /// as-is to docker-compose, without verifying any arguments.
    ///
    pub fn dc(_ctx: &Context, _env: &M2Vars, trailing: Vec<String>, dc: DcTasks) -> Vec<Task> {
        let after: Vec<String> = trailing.into_iter().skip(1).collect();
        vec![dc.cmd_task(after)]
    }

    pub fn node(_ctx: &Context, _env: &M2Vars, trailing: Vec<String>, dc: DcTasks) -> Vec<Task> {
        let dc_command = format!(r#"run {}"#, trailing.join(" "));
        vec![dc.cmd_task(vec![dc_command])]
    }

    pub fn composer(ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let container_name = PhpContainer::from_ctx(&ctx).name;
        let exec_command = format!(
            r#"docker exec -it -u www-data {container_name} {trailing_args}"#,
            container_name = container_name,
            trailing_args = trailing.join(" ")
        );
        vec![Task::simple_command(exec_command)]
    }

    pub fn mage(ctx: &Context, trailing: Vec<String>) -> Vec<Task> {
        let container_name = PhpContainer::from_ctx(&ctx).name;
        let full_command = format!(
            r#"docker exec -it -u www-data -e COLUMNS="{width}" -e LINES="{height}" {container_name} ./bin/magento {trailing_args}"#,
            width = ctx.term.width,
            height = ctx.term.height,
            container_name = container_name,
            trailing_args = trailing.into_iter().skip(1).collect::<Vec<String>>().join(" ")
        );
        vec![Task::simple_command(full_command)]
    }

    pub fn npm(ctx: &Context, _env: &M2Vars, trailing: Vec<String>, dc: DcTasks) -> Vec<Task> {
        let dc_command = format!(
            r#"run --workdir {work_dir} {service} {trailing_args}"#,
            work_dir = path_buf_to_string(&PathBuf::from("/var/www").join(ctx.npm_path.clone())),
            service = "node",
            trailing_args = trailing.join(" ")
        );
        vec![dc.cmd_task(vec![dc_command])]
    }
}

pub fn commands() -> Vec<(String, String)> {
    vec![
        (
            M2PassThru::Composer,
            "[M2] Run composer commands with the correct user",
        ),
        (M2PassThru::Npm, "[M2] Run npm commands"),
        (M2PassThru::Dc, "[M2] Run docker-compose commands"),
        (M2PassThru::Node, "[M2] Run commands in the node container"),
        (
            M2PassThru::M,
            "[M2] Execute ./bin/magento commands inside the PHP container",
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
