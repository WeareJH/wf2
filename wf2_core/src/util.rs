use std::process::{Command, Output};
use std::{collections::HashMap, path::PathBuf};

pub fn path_buf_to_string(pb: &PathBuf) -> String {
    pb.to_string_lossy().to_string()
}

pub fn replace_env(env: HashMap<String, String>, input: &[u8]) -> Vec<u8> {
    use regex::{Captures, Regex};
    let re = Regex::new(r"\$\{(.+?)}").unwrap();
    re.replace_all(
        std::str::from_utf8(input).unwrap(),
        |caps: &Captures| match env.get(&caps[1]) {
            Some(out) => out.clone(),
            None => String::from("..."),
        },
    )
    .to_string()
    .into()
}

pub fn has_pv() -> Option<String> {
    let mut cmd = Command::new("which");
    cmd.arg("pv");
    match cmd.output() {
        Ok(Output { status, stdout, .. }) => match status.code() {
            Some(0) => std::str::from_utf8(&stdout)
                .map(|s| s.trim().to_string())
                .ok(),
            _ => None,
        },
        Err(..) => None,
    }
}

#[test]
fn test_eject() {
    let dc_bytes = b"wf__${WF2_PWD}__unison";
    let mut hm = HashMap::new();
    hm.insert("WF2_PWD".to_string(), "/shane".to_string());
    let output = replace_env(hm, dc_bytes);
    println!("{:?}", output);
}
