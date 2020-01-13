use ansi_term::Colour::{Cyan, Green, Yellow};
use std::process::{Command, Stdio};

pub fn output(left: impl Into<String>, right: impl Into<String>) -> String {
    format!(
        "{}: {}",
        Yellow.paint(left.into()),
        Green.paint(right.into())
    )
}

pub fn output_left(left: impl Into<String>, right: impl Into<String>) -> String {
    format!("{}: {}", Yellow.paint(left.into()), right.into(),)
}

pub fn file(left: impl Into<String>) -> String {
    format!("{}", Cyan.paint(left.into()))
}

///
/// Capture git diff output with color.
///
pub fn git_diff_output(
    left: impl Into<String>,
    right: impl Into<String>,
) -> Result<String, failure::Error> {
    let mut child_process = Command::new("sh");

    let cmd = format!(
        "git -c color.ui=always --no-pager diff --no-index {left} {right}",
        left = left.into(),
        right = right.into(),
    );

    child_process.arg("-c").arg(cmd);
    child_process.stdin(Stdio::inherit());
    child_process.stdout(Stdio::piped());

    let output = child_process.output()?;
    let output = String::from_utf8(output.stdout)?
        .lines()
        .skip(2)
        .map(String::from)
        .collect::<Vec<String>>()
        .join("\n");

    Ok(output)
}
