use crate::context::Context;
use std::collections::HashMap;

pub trait Vars<T: Sized> {
    fn from_ctx(ctx: &Context) -> Result<T, String>;
}
