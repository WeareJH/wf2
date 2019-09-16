use crate::context::Context;
use std::path::PathBuf;

pub trait File<T: Sized> {
    fn from_ctx(ctx: &Context) -> Result<T, String>;
    fn file_path(&self) -> PathBuf;
    fn bytes(&self) -> Vec<u8>;
}
