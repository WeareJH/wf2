use ansi_term::Colour::{Cyan, Green, Yellow};
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
