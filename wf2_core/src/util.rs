use std::{collections::HashMap, path::PathBuf};

pub fn path_buf_to_string(pb: &PathBuf) -> String {
    pb.to_string_lossy().to_string()
}

pub fn two_col(commands: Vec<(String, String)>) -> String {
    match commands.clone().get(0) {
        Some(_t) => {
            let longest = commands.iter().fold(
                commands[0].clone(),
                |(prev_name, prev_desc), (name, help)| {
                    if name.len() > prev_name.len() {
                        (name.to_string(), help.to_string())
                    } else {
                        (prev_name, prev_desc)
                    }
                },
            );
            let longest = longest.0.len();
            let lines = commands
                .into_iter()
                .map(|(name, help)| {
                    let cur_len = name.len();
                    let diff = longest - cur_len;
                    let diff = match longest - cur_len {
                        0 => 4,
                        _ => diff + 4,
                    };
                    format!(
                        "    {name}{:diff$}{help}",
                        " ",
                        name = name,
                        diff = diff,
                        help = help
                    )
                })
                .collect::<Vec<String>>();
            lines.join("\n")
        }
        None => String::from(""),
    }
}

#[test]
fn test_get_after_help_lines() {
    let actual = two_col(vec![
        (String::from("npm"), String::from("help string")),
        (
            String::from("composer"),
            String::from("another help string"),
        ),
        (String::from("m"), String::from("another help string")),
    ]);
    assert_eq!(
        actual,
        "    npm         help string
    composer    another help string
    m           another help string"
    );
}
