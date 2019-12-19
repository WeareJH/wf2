use crate::context::Context;

pub trait Vars<T: Sized> {
    fn from_ctx(ctx: &Context) -> Result<T, failure::Error>;
}
