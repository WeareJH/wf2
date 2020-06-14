use crate::context::Context;
use crate::dc_tasks::DcTasks;
use crate::recipes::m2::subcommands::composer::composer;

use crate::subcommands::dc::dc_passthru;
use crate::task::Task;
use std::cmp;

///
/// Enum to represent all of the possible pass-thru commands available
///
#[derive(Debug, Clone)]
pub enum WpPassThru {
    Composer,
    Dc,
    WpCli,
}

impl WpPassThru {
    ///
    /// Passthru command names
    ///
    const COMPOSER: &'static str = "composer";
    const DC: &'static str = "dc";
    const WP_CLI: &'static str = "wp";

    ///
    /// Helper method for converting an enum member to a String
    ///
    pub fn name(&self) -> String {
        match self {
            WpPassThru::Composer => WpPassThru::COMPOSER,
            WpPassThru::Dc => WpPassThru::DC,
            WpPassThru::WpCli => WpPassThru::WP_CLI,
        }
        .to_string()
    }
    pub fn resolve_cmd(
        ctx: &Context,
        cmd: String,
        trailing: &[String],
        dc: DcTasks,
    ) -> Option<Vec<Task>> {
        match cmd {
            ref x if *x == WpPassThru::Dc => {
                let res = dc_passthru(ctx, trailing);
                Some(res.unwrap_or_else(Task::task_err_vec))
            }
            ref x if *x == WpPassThru::WpCli => Some(wp_cli_passthru(trailing, dc)),
            ref x if *x == WpPassThru::Composer => Some(composer(&ctx, trailing)),
            _ => None,
        }
    }

    pub fn commands() -> Vec<(String, String)> {
        vec![
            (
                WpPassThru::Composer,
                "[wp] Run composer commands with the correct user",
            ),
            (WpPassThru::Dc, "[wp] Run docker-compose commands"),
            (WpPassThru::WpCli, "[wp] Run Wordpress CLI commands"),
        ]
        .into_iter()
        .map(|(name, help)| (name.into(), help.into()))
        .collect()
    }
}

pub fn wp_cli_passthru(trailing: &[String], dc: DcTasks) -> Vec<Task> {
    let dc_command = format!(r#"run --no-deps {}"#, trailing.join(" "));
    vec![dc.cmd_task(vec![dc_command])]
}

impl From<WpPassThru> for String {
    fn from(m2p: WpPassThru) -> Self {
        m2p.name()
    }
}

///
/// Allow a comparison to String
///
/// ```
/// use wf2_core::recipes::wp::pass_thru::WpPassThru;
/// assert_eq!(true, WpPassThru::Composer == String::from("composer"));
/// ```
///
impl cmp::PartialEq<String> for WpPassThru {
    fn eq(&self, other: &String) -> bool {
        self.name() == *other
    }
}

impl cmp::PartialEq<WpPassThru> for String {
    fn eq(&self, other: &WpPassThru) -> bool {
        *self == other.name()
    }
}
