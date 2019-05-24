use snailquote::{escape, unescape};
use std::collections::btree_map::BTreeMap;
use std::str;
use users::{get_current_gid, get_current_uid};

///
/// Use the base template & append custom bits
///
pub fn create_env(domain: &str) -> Vec<u8> {
    let env_content = include_bytes!("./templates/m2/.env");
    let mut new_env = get_env_store(env_content);

    let current_uid = get_current_uid();
    let current_gid = get_current_gid();
    new_env.insert("HOST_UID".into(), current_uid.to_string());
    new_env.insert("HOST_GID".into(), current_gid.to_string());

    new_env.insert("MAGE_HOST".into(), format!("http://{}", domain));
    new_env.insert("PHP_IDE_CONFIG".into(), format!("serverName={}", domain));

    print(new_env)
}

///
/// Just a map of key/value pairs to represent the env
///
pub fn get_env_store(bytes: &[u8]) -> BTreeMap<String, String> {
    let mut store = BTreeMap::new();
    let values = bytes.split(|&x| x == b'\n').flat_map(parse_line);

    for (key, value) in values {
        store.insert(key, value);
    }
    store
}

///
/// Take the map and convert to a Vec<u8> for writing to disk
///
pub fn print(store: BTreeMap<String, String>) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(1024);
    for (key, value) in &store {
        buffer.extend_from_slice(key.as_bytes());
        buffer.push(b'=');
        // The value may contain space and need to be quoted
        let v = escape(value.as_str()).into_owned();
        buffer.extend_from_slice(v.as_bytes());
        buffer.push(b'\n');
    }

    buffer
}

///
/// Parse a single line, copy/pasted from another lib
///
fn parse_line(entry: &[u8]) -> Option<(String, String)> {
    str::from_utf8(entry).ok().and_then(|l| {
        let line = l.trim();
        // Ignore comment line
        if line.starts_with('#') {
            return None;
        }
        let vline = line.as_bytes();
        vline.iter().position(|&x| x == b'=').and_then(|pos| {
            str::from_utf8(&vline[..pos]).ok().and_then(|x| {
                str::from_utf8(&vline[pos + 1..]).ok().and_then(|right| {
                    // The right hand side value can be a quoted string
                    unescape(right).ok().map(|y| (x.to_owned(), y))
                })
            })
        })
    })
}
