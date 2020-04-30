use crate::output::git_diff_output;
use crate::{
    conditions::{file_present::FilePresent, files_differ::FilesDiffer, question::Question},
    context::Context,
    task::Task,
};
use ansi_term::Colour::{Cyan, Green};
use std::path::PathBuf;

///
/// This represents tasks related to the env.php file.
///
pub struct EnvPhp {
    pub env: PathBuf,
    pub env_dist: PathBuf,
}

impl EnvPhp {
    ///
    /// This is the path that's required for magento to run correctly
    ///
    pub const ENV: &'static str = "app/etc/env.php";
    ///
    /// This is the file that *should* be part of the project repo
    ///
    pub const ENV_DIST: &'static str = "app/etc/env.php.dist";
    ///
    /// Generate file paths based on the CWD
    ///
    pub fn from_ctx(ctx: &Context) -> EnvPhp {
        EnvPhp {
            env: ctx.cwd.join(EnvPhp::ENV),
            env_dist: ctx.cwd.join(EnvPhp::ENV_DIST),
        }
    }
    ///
    /// This task first checks if the following conditions are met
    ///     1. is env.php missing
    ///     2. is env.php.dist present
    ///
    /// If both are 'true', it will copy env.php.dist -> env.php
    ///
    /// If either evaluate to false, (eg: if both files are present)
    /// then it will instead run the comparison check if the files
    /// have different content by deferring to [`EnvPhp::env_php_comparison_task`]
    ///
    pub fn missing_task(ctx: &Context) -> Task {
        let env = EnvPhp::from_ctx(ctx);
        Task::conditional(
            vec![
                Box::new(FilePresent::new(env.env.clone(), true)),
                Box::new(FilePresent::new(env.env_dist.clone(), false)),
            ],
            vec![env.copy_dist(), env.copy_dist_msg()],
            vec![EnvPhp::comparison_task(&ctx)],
            Some("copy env.php.dist -> env.php"),
        )
    }
    ///
    /// Check if the env.php.dist differs from an existing
    /// env.php file.
    ///
    /// This can happen when changes are made to env.php.dist
    /// which is a file that's part of version control and
    /// it needs rolling out to users
    ///
    pub fn comparison_task(ctx: &Context) -> Task {
        let env = EnvPhp::from_ctx(ctx);
        Task::conditional(
            vec![
                // Does the env.php file exist?
                Box::new(FilePresent::new(&env.env, false)),
                // Does the env.php.dist file exist?
                Box::new(FilePresent::new(&env.env_dist, false)),
                // Do those files differ?
                Box::new(FilesDiffer::new(&env.env, &env.env_dist)),
                // Does the user want to copy env.php.dist -> env.php
                Box::new(env.copy_confirm()),
            ],
            vec![env.copy_dist(), env.copy_dist_msg(), env.copy_warning()],
            vec![],
            Some("env.php.dist comparison with env.php"),
        )
    }
    ///
    /// Perform the copy
    ///
    pub fn copy_dist(&self) -> Task {
        Task::file_clone(&self.env_dist, &self.env)
    }
    ///
    /// Print the message about the copy
    ///
    pub fn copy_dist_msg(&self) -> Task {
        Task::notify_prefixed(format!(
            "Copied {} to {}",
            Cyan.paint(EnvPhp::ENV_DIST),
            Cyan.paint(EnvPhp::ENV)
        ))
    }
    ///
    /// Present a warning about the need for
    /// running app:config:import
    ///
    pub fn copy_warning(&self) -> Task {
        let warning = format!(
            "You will need to run `{cmd}` once everything has started",
            cmd = Cyan.paint("wf2 m app:config:import")
        );
        Task::notify_prefixed(warning)
    }
    ///
    /// Confirm if the user actually wants to override the file
    ///
    pub fn copy_confirm(&self) -> Question {
        let prefix = Green.paint("[wf2 info]");
        let left = Cyan.paint(EnvPhp::ENV);
        let right = Cyan.paint(EnvPhp::ENV_DIST);
        let question = git_diff_output(EnvPhp::ENV_DIST, EnvPhp::ENV)
            .map(|diff| {
                format!(
                    "{diff}\n\n{prefix}: Your local {left} doesn't match {right} (diff above), override?",
                    prefix = prefix,
                    diff = diff,
                    left = left,
                    right = right,
                )
            })
            .unwrap_or_else(|_| format!(
                "{prefix}: Your local {left} doesn't match {right}, override?",
                prefix = prefix,
                left = left,
                right = right,
            ));

        Question::new(question)
    }
}
