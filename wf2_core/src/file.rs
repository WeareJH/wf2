use crate::context::Context;
use crate::task::Task;
use std::path::PathBuf;

pub trait File<T: Sized> {
    fn from_ctx(ctx: &Context) -> Result<T, String>;
    fn file_path(&self) -> PathBuf;
    fn bytes(&self) -> Vec<u8>;
    fn write(&self) -> Task {
        Task::file_write(self.file_path(), "Task::file_write", self.bytes())
    }
}
