use crate::{
    conditions::{file_present::FilePresent, files_differ::FilesDiffer, question::Question},
    context::Context,
    task::Task,
};
use ansi_term::Colour::{Cyan, Green};

const LEFT: &'static str = "app/etc/env.php";
const RIGHT: &'static str = "app/etc/env.php.dist";

pub fn env_php_task(ctx: &Context) -> Task {
    let left_abs = ctx.cwd.join(LEFT);
    let right_abs = ctx.cwd.join(RIGHT);
    let prefix = Green.paint("[wf2 info]");
    let question = format!(
        "{prefix}: Your local {left} doesn't match {right}, override?",
        prefix = prefix,
        left = Cyan.paint(LEFT),
        right = Cyan.paint(RIGHT)
    );
    let warning = format!(
        "{prefix}: You will need to run `{cmd}` once everything has started",
        prefix = prefix,
        cmd = Cyan.paint("wf2 m app:config:import")
    );
    Task::conditional(
        vec![
            Box::new(FilePresent::new(left_abs.clone())),
            Box::new(FilePresent::new(right_abs.clone())),
            Box::new(FilesDiffer::new(left_abs.clone(), right_abs.clone())),
            Box::new(Question::new(question)),
        ],
        vec![
            Task::file_clone(right_abs, left_abs),
            Task::notify(format!(
                "{}: Copied {} to {}",
                prefix,
                Cyan.paint(RIGHT),
                Cyan.paint(LEFT)
            )),
            Task::notify(format!("{}", warning)),
        ],
        Some("env.php.dist comparison with env.php"),
    )
}
