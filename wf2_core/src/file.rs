use crate::context::Context;
use crate::task::Task;
use crate::util::path_buf_to_string;
use std::path::PathBuf;

pub trait File<T: Sized> {
    ///
    /// A short, one-line description of this files purpose
    ///
    const DESCRIPTION: &'static str;

    ///
    /// The relative (to CWD) path for this file
    ///
    const OUTPUT_PATH: &'static str;

    fn from_ctx(ctx: &Context) -> Result<T, failure::Error>;
    fn file_path(&self) -> PathBuf;
    fn bytes(&self) -> Vec<u8> {
        vec![]
    }
    fn description(&self) -> String {
        format!("Recipe file: {}", self.file_path().display())
    }
    fn write_task(&self) -> Task {
        Task::file_write(self.file_path(), Self::DESCRIPTION, self.bytes())
    }
    fn exists_task(&self) -> Task {
        Task::file_exists(self.file_path(), Self::DESCRIPTION)
    }
    fn dir(&self) -> PathBuf {
        let mut pb = self.file_path();
        pb.pop();
        pb
    }
    fn dir_string(&self) -> String {
        let mut pb = self.file_path();
        pb.pop();
        path_buf_to_string(&pb)
    }
    fn file_path_string(&self) -> String {
        path_buf_to_string(&self.file_path())
    }
}
