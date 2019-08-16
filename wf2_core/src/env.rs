use crate::context::Context;
use std::collections::HashMap;
use std::path::PathBuf;

pub trait Env<T: Sized> {
    fn from_ctx(ctx: &Context) -> Result<T, String>;
    fn content(&self) -> HashMap<String, String>;
    fn file_path(&self) -> PathBuf;
}
