use crate::context::Context;
use snailquote::{escape, unescape};
use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str;

pub trait Env<T: Sized> {
    fn from_ctx(ctx: &Context) -> Result<T, String>;
    fn content(&self) -> HashMap<String, String>;
    fn file_path(&self) -> PathBuf;
}
